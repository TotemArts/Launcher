#![windows_subsystem="console"]
#![warn(clippy::multiple_crate_versions)]

extern crate native_tls;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio;
extern crate tokio_tls;
extern crate tokio_reactor;
extern crate futures;
#[macro_use] extern crate sciter;
extern crate renegadex_patcher;
extern crate ini;
extern crate single_instance;
extern crate socket2;
extern crate rand;
extern crate unzip;
extern crate dirs;
extern crate tower;
extern crate runas;
extern crate sha2;
extern crate hex;

use std::sync::{Arc,Mutex};

use sciter::Value;

use socket2::*;

use renegadex_patcher::{Downloader,Update, traits::Error};
use ini::Ini;
use single_instance::SingleInstance;
use std::io::Write;
use crate::hyper::body::HttpBody; 
use hyper::client::{Client, HttpConnector};
use hyper::http::header::{HeaderMap, HeaderName, HeaderValue};
use hyper::client::connect::dns::Name;
use sha2::{Sha256, Digest};

use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::task::Poll;
use std::future::Future;


#[derive(Debug, Clone)]
pub struct SocketAddrs {
  inner: std::vec::IntoIter<std::net::SocketAddr>
}

impl PartialEq for SocketAddrs {
  fn eq(&self, other: &SocketAddrs) -> bool {
    self.inner.as_slice() == other.inner.as_slice()
  }
}

impl From<Vec<std::net::SocketAddr>> for SocketAddrs {
  fn from(other: Vec<std::net::SocketAddr>) -> Self {
    SocketAddrs {
      inner: other.into_iter()
    }
  }
}

impl ToSocketAddrs for SocketAddrs {
  type Iter = std::vec::IntoIter<std::net::SocketAddr>;
  fn to_socket_addrs(&self) -> std::io::Result<std::vec::IntoIter<std::net::SocketAddr>> {
    Ok(self.inner.clone())
  }
}

impl Iterator for SocketAddrs {
  type Item = std::net::IpAddr;

  fn next(&mut self) -> Option<Self::Item> {
      self.inner.next().map(|sa| sa.ip())
  }
}


#[derive(Clone)]
pub struct ResolverService {
  pub socket_addrs: SocketAddrs
}

impl ResolverService {
  pub fn new(socket_addrs: SocketAddrs) -> Self {
    ResolverService {
      socket_addrs
    }
  }
}

impl tower::Service<Name> for ResolverService {
  type Response = SocketAddrs;
  type Error = Error;
  // We can't "name" an `async` generated future.
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send >>;

  fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
      // This connector is always ready, but others might not be.
      Poll::Ready(Ok(()))
  }

  fn call(&mut self, _: Name) -> Self::Future {
    let socket_addrs = self.socket_addrs.clone();
    let fut = async move { 
      Ok(socket_addrs) 
    };
    Box::pin(fut)
  }
}

/// The current launcher's version
static VERSION: &str = env!("CARGO_PKG_VERSION");

/// Structure for Sciter event handling.
struct Handler {
  /// The reference to the back-end library which is responsible for downloading and updating the game.
  patcher: Arc<Mutex<Downloader>>,
  /// The configuration file for the launcher.
  conf: Arc<Mutex<ini::Ini>>
}

impl Handler {
  /// Check if there are game updates available, makes use of caching.
  fn check_update(&self, done: sciter::Value, error: sciter::Value) {
    {
      let progress = self.patcher.clone().lock().expect(concat!(file!(),":",line!())).get_progress();
      let update = &progress.lock().expect(concat!(file!(),":",line!())).update.clone();
      match update {
        Update::UpToDate => {
          std::thread::spawn(move || {done.call(None, &make_args!("up_to_date"), None).expect(concat!(file!(),":",line!()));});
          return;
        },
        Update::Full => {
          std::thread::spawn(move || {done.call(None, &make_args!("full"), None).expect(concat!(file!(),":",line!()));});
          return;
        },
        Update::Resume => {
          std::thread::spawn(move || {done.call(None, &make_args!("resume"), None).expect(concat!(file!(),":",line!()));});
          return;
        },
        Update::Delta => {
          std::thread::spawn(move || {done.call(None, &make_args!("update"), None).expect(concat!(file!(),":",line!()));});
          return;
        },
        Update::Unknown => {}
      }
    }
    let patcher = self.patcher.clone();
		std::thread::spawn(move || {
      let check_update = || -> Result<(), Error> {
        let update_available : Update;
        {
          let mut patcher = patcher.lock().expect(concat!(file!(),":",line!()));
          patcher.retrieve_mirrors().expect(concat!(file!(),":",line!()));
          update_available = patcher.update_available().expect(concat!(file!(),":",line!()));
        }
        match update_available {
          Update::UpToDate => {
            std::thread::spawn(move || {done.call(None, &make_args!("up_to_date"), None).expect(concat!(file!(),":",line!()));});
          },
          Update::Full => {
            std::thread::spawn(move || {done.call(None, &make_args!("full"), None).expect(concat!(file!(),":",line!()));});
          },
          Update::Resume => {
            std::thread::spawn(move || {done.call(None, &make_args!("resume"), None).expect(concat!(file!(),":",line!()));});
          },
          Update::Delta => {
            std::thread::spawn(move || {done.call(None, &make_args!("patch"), None).expect(concat!(file!(),":",line!()));});
          },
          Update::Unknown => {
            eprintln!("Update::Unknown");
          }
        };
        Ok(())
		  };
      let result : Result<(),Error> = check_update();
      if let Err(err) = result {
        eprintln!("{:#?}", &err);
        std::thread::spawn(move || {error.call(None, &make_args!(err.to_string()), None).expect(concat!(file!(),":",line!()));});
      }
    });
  }

  /// Starts the downloading of the update/game
  fn start_download(&self, callback: sciter::Value, callback_done: sciter::Value, error: sciter::Value) {
    let progress = self.patcher.clone().lock().expect(concat!(file!(),":",line!())).get_progress();
		std::thread::spawn(move || {
      let mut not_finished = true;
      let mut last_download_size : u64 = 0;
      while not_finished {
        std::thread::sleep(std::time::Duration::from_millis(500));
        {
          let progress_locked = progress.lock().expect(concat!(file!(),":",line!()));
          let me : Value = format!(
            "{{\"hash\": [{},{}],\"download\": [{},{}],\"patch\": [{},{}],\"download_speed\": {}}}",
            progress_locked.hashes_checked.0,
            progress_locked.hashes_checked.1,
            progress_locked.download_size.0/10_000,
            progress_locked.download_size.1/10_000,
            progress_locked.patch_files.0,
            progress_locked.patch_files.1,
            (progress_locked.download_size.0 - last_download_size) as f64 / 500_000.0
          ).parse().expect(concat!(file!(),":",line!()));
          last_download_size = progress_locked.download_size.0;
          not_finished = !progress_locked.finished_patching;
          drop(progress_locked);
          let callback_clone = callback.clone();
          std::thread::spawn(move || {callback_clone.call(None, &make_args!(me), None).expect(concat!(file!(),":",line!()));});
        }
      }
		});
    let patcher = self.patcher.clone();
    std::thread::spawn(move || {
      let result : Result<(), renegadex_patcher::traits::Error>;
      {
        let mut locked_patcher = patcher.lock().expect(concat!(file!(),":",line!()));
        locked_patcher.rank_mirrors().expect(concat!(file!(),":",line!()));
        locked_patcher.poll_progress();
        result = locked_patcher.download();
      }
      match result {
        Ok(()) => {
          println!("Calling download done");
          std::thread::spawn(move || {callback_done.call(None, &make_args!(false,false), None).expect(concat!(file!(),":",line!()));});
        },
        Err(e) => {
          eprintln!("{:#?}", &e);
          std::thread::spawn(move || {error.call(None, &make_args!(e.to_string()), None).expect(concat!(file!(),":",line!()));});
        }
      };
    });
  }

  /// Removes files inside of the subdirectories that are not part of the instructions.json
  fn remove_unversioned(&self, callback_done: sciter::Value, error: sciter::Value) {
    let patcher = self.patcher.clone();
    std::thread::spawn(move || {
      let result : Result<(), renegadex_patcher::traits::Error>;
      {
        let mut locked_patcher = patcher.lock().expect(concat!(file!(),":",line!()));
        locked_patcher.rank_mirrors().expect(concat!(file!(),":",line!()));
        result = locked_patcher.remove_unversioned();
      }
      match result {
        Ok(()) => {
          println!("Calling remove unversioned done");
          std::thread::spawn(move || {callback_done.call(None, &make_args!("validate"), None).expect(concat!(file!(),":",line!()));});
        },
        Err(e) => {
          eprintln!("Error in remove_unversioned(): {:#?}", &e);
          std::thread::spawn(move || {error.call(None, &make_args!(e.to_string()), None).expect(concat!(file!(),":",line!()));});
        }
      };
    });
  }

  /// Retrieve the playername
  fn get_playername(&self) -> String {
    let conf_unlocked = self.conf.clone();
    let conf = conf_unlocked.lock().expect(concat!(file!(),":",line!()));
    let section = conf.section(Some("RenX_Launcher".to_owned())).expect(concat!(file!(),":",line!()));
    section.get("PlayerName").expect(concat!(file!(),":",line!())).to_string()
  }

  /// Set the playername
  fn set_playername(&self, username: sciter::Value) {
    let conf_unlocked = self.conf.clone();
    let mut conf = conf_unlocked.lock().expect(concat!(file!(),":",line!()));
    let mut section = conf.with_section(Some("RenX_Launcher".to_owned()));
    section.set("PlayerName", username.as_string().expect(concat!(file!(),":",line!())));

    let mut config_directory = dirs::config_dir().unwrap();
    config_directory.push("Renegade X");
    std::fs::create_dir_all(&config_directory).expect("Creation of config-directory went wrong!");
    config_directory.push("Renegade X Launcher.ini");

    conf.write_to_file(config_directory.to_str().unwrap()).expect(concat!(file!(),":",line!()));
  }

  /// Get Server List as plain text
  fn get_servers(&self, callback: sciter::Value) {
    std::thread::spawn(move || {
      let mut rt = tokio::runtime::Builder::new().basic_scheduler().enable_time().enable_io().build().unwrap();
      let result = rt.enter(|| {
        rt.spawn(async move {
          let url = "https://serverlist.renegade-x.com/servers.jsp?id=launcher".parse::<hyper::Uri>()?;
          let https = hyper_tls::HttpsConnector::new();
          let client = hyper::Client::builder().build::<_, hyper::Body>(https);
          let req = hyper::Request::builder();
          if let Some(host) = url.host() {
            let req = req.uri(url.clone()).header("host", host).header("User-Agent", format!("RenX-Launcher ({})", VERSION));
            let req = req.body(hyper::Body::empty())?;
            let res = client.request(req).await?;
            if res.status() == 200 || res.status() == 206 {
              let buffer = hyper::body::to_bytes(res).await?;
              std::thread::spawn(move || {
                let text : Value = ::std::str::from_utf8(&buffer).expect("Expected an utf-8 string").parse().expect(concat!(file!(),":",line!()));
                callback.call(None, &make_args!(text), None).expect(concat!(file!(),":",line!()));
              });
            }
          }
          Ok::<(), Error>(())
        })
      });
      let _ = rt.block_on(result).unwrap();
    });
  }

  /// Get ping of server
  fn get_ping(&self, server: sciter::Value, callback: sciter::Value) {
    std::thread::spawn(move || {
      let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4())).expect(concat!(file!(),":",line!(),": New socket"));
      let server_string = server.as_string().expect(&format!("Couldn't cast server \"{:?}\" to string", &server));
      let mut server_socket = server_string.to_socket_addrs().expect(&format!("Couldn't unwrap socket address of server \"{}\"", &server_string));
      let sock_addr = server_socket.next().expect(&format!("No Sockets found for DNS name \"{}\"", &server_string)).into();
      let start_time = std::time::Instant::now();
      socket.connect_timeout(&sock_addr, std::time::Duration::from_millis(500)).expect(concat!(file!(),":",line!()));
      let mut code = [0x08, 0x00, 0x00, 0x00, rand::random::<u8>(), rand::random::<u8>(), 0x00, 0x01, 0x02, 0x59, 0x9d, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x98, 0x61, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37];
      let mut checksum : u64 = 0;
      for i in (0..code.len()).step_by(2) {
        checksum = checksum.wrapping_add(u16::from_be_bytes([code[i],code[i+1]]) as u64);
      }
      if code.len()%2>0 {
        checksum = checksum.wrapping_add(code[code.len()-1] as u64);
      }
      while checksum.wrapping_shr(16) != 0 {
        checksum = (checksum & 0xffff) + checksum.wrapping_shr(16);
      }
      checksum ^= 0xffff;
      let checksum = (checksum as u16).to_be_bytes();
      code[2] = checksum[0];
      code[3] = checksum[1];
      socket.send(&code).expect(concat!(file!(),":",line!()));
      let mut buf : [u8; 100] = [0; 100];
      socket.set_read_timeout(Some(std::time::Duration::from_millis(500))).expect(concat!(file!(),":",line!()));
      let result = socket.recv(&mut buf);
      let elapsed = start_time.elapsed().as_millis() as i32;
      if result.is_ok() && buf[36..36+48] == code[16..] {
        std::thread::spawn(move || {callback.call(None, &make_args!(server, elapsed), None).expect(concat!(file!(),":",line!()));});
      }
    });
  }

  /// Get the installed game's version
  fn get_game_version(&self) -> String {
    let conf = self.conf.lock().expect(concat!(file!(),":",line!()));
    let section = conf.section(Some("RenX_Launcher".to_owned())).expect(concat!(file!(),":",line!()));
    let game_location = section.get("GameLocation").expect(concat!(file!(),":",line!())).clone();
    match Ini::load_from_file(format!("{}/UDKGame/Config/DefaultRenegadeX.ini", game_location)) {
      Ok(conf) => {
        let section = conf.section(Some("RenX_Game.Rx_Game".to_owned())).expect(concat!(file!(),":",line!()));
        section.get("GameVersion").expect(concat!(file!(),":",line!())).to_string()
      },
      Err(_e) => {
        "Not installed".to_string()
      }
    }
  }

  /// Launch the game, if server variable it's value is "", then the game will be launched to the menu.
  fn launch_game(&self, server: Value, done: Value, error: Value) {
    let conf = self.conf.lock().expect(concat!(file!(),":",line!()));
    let section = conf.section(Some("RenX_Launcher".to_owned())).expect(concat!(file!(),":",line!()));
    let game_location = section.get("GameLocation").expect(concat!(file!(),":",line!())).clone();
    let playername = section.get("PlayerName").expect(concat!(file!(),":",line!())).clone();
    let startup_movie_disabled = section.get("skipMovies").expect(concat!(file!(),":",line!())).clone() == "true";
    let bit_version = if section.get("64-bit-version").expect(concat!(file!(),":",line!())).clone() == "true" { "64" } else { "32" };
    drop(conf);
    std::thread::spawn(move || {
      let server = server.as_string().expect(concat!(file!(),":",line!()));
      let mut args = vec![];
      match server.as_str() {
        "" => {},
        _ => args.push(server)
      };
      args.push(format!("-ini:UDKGame:DefaultPlayer.Name={}", playername));
      if startup_movie_disabled {
        args.push("-nomoviestartup".to_string());
      }
      match std::process::Command::new(format!("{}/Binaries/Win{}/UDK.exe", game_location, bit_version))
                                     .args(&args)	
                                     .stdout(std::process::Stdio::piped())
                                     .stderr(std::process::Stdio::inherit())
                                     .spawn() {
        Ok(mut child) => {
          let output = child.wait().expect("Failed to wait on game-instance to finish");
          if output.success() {
            std::thread::spawn(move || {done.call(None, &make_args!(), None).expect(concat!(file!(),":",line!()));});
          } else {
            //eprintln!("{:#?}", output.unwrap_err());
            std::thread::spawn(move || {error.call(None, &make_args!(format!("The game exited in a crash: {}", output.code().expect(concat!(file!(),":",line!())))), None).expect(concat!(file!(),":",line!()));});
          }
        },
        Err(e) => {
          eprintln!("Failed to create child: {}", &e);
          std::thread::spawn(move || {error.call(None, &make_args!(format!("Failed to open game: {}", &e)), None).expect(concat!(file!(),":",line!()));});
        }
      };
    });
  }

  /// Gets the setting from the launchers configuration file.
  fn get_setting(&self, setting: sciter::Value) -> String {
    let conf_unlocked = self.conf.clone();
    let conf = conf_unlocked.lock().expect(concat!(file!(),":",line!()));
    let section = conf.section(Some("RenX_Launcher".to_owned())).expect(concat!(file!(),":",line!()));
    section.get(&setting.as_string().expect(concat!(file!(),":",line!()))).expect(concat!(file!(),":",line!())).to_string()
  }

  /// Sets the setting in the launchers configuration file.
  fn set_setting(&self, setting: sciter::Value, value: sciter::Value) {
    let conf_unlocked = self.conf.clone();
    let mut conf = conf_unlocked.lock().expect(concat!(file!(),":",line!()));
    let mut section = conf.with_section(Some("RenX_Launcher".to_owned()));
    section.set(setting.as_string().expect(concat!(file!(),":",line!())), value.as_string().expect(concat!(file!(),":",line!())));
    
    let mut config_directory = dirs::config_dir().unwrap();
    config_directory.push("Renegade X");
    std::fs::create_dir_all(&config_directory).expect("Creation of config-directory went wrong!");
    config_directory.push("Renegade X Launcher.ini");

    conf.write_to_file(config_directory.to_str().unwrap()).expect(concat!(file!(),":",line!()));
  }

  /// Get the current launcher version
  fn get_launcher_version(&self) -> &str {
    VERSION
  }

  /// Checks if the launcher is up to date
  fn check_launcher_update(&self, callback: Value) {
    let launcher_info_option = self.patcher.lock().expect(concat!(file!(),":",line!())).get_launcher_info();
    if let Some(launcher_info) = launcher_info_option {
      if VERSION != launcher_info.version_name && !launcher_info.prompted {
        std::thread::spawn(move || {callback.call(None, &make_args!(launcher_info.version_name), None).expect(concat!(file!(),":",line!()));});
      } else {
        std::thread::spawn(move || {callback.call(None, &make_args!(Value::null()), None).expect(concat!(file!(),":",line!()));});
      }
    } else {
      let patcher = self.patcher.clone();
      std::thread::spawn(move || {
        let mut patcher = patcher.lock().expect(concat!(file!(),":",line!()));
        patcher.retrieve_mirrors().expect(concat!(file!(),":",line!()));
        let launcher_info_option = patcher.get_launcher_info();
        drop(patcher);
        if let Some(launcher_info) = launcher_info_option {
          if VERSION != launcher_info.version_name && !launcher_info.prompted {
            std::thread::spawn(move || {callback.call(None, &make_args!(launcher_info.version_name), None).expect(concat!(file!(),":",line!()));});
          } else {
            std::thread::spawn(move || {callback.call(None, &make_args!(Value::null()), None).expect(concat!(file!(),":",line!()));});
          }
        }
      });
    }
  }

  fn install_redists(&self, done: Value, error_callback: Value) {
    if let Some(mut cache_dir) = dirs::cache_dir() {
      let patcher = self.patcher.clone();
      // Spawn thread, to not block the main process.
      std::thread::spawn(move || {
        cache_dir.set_file_name("UE3Redist.exe");
        let file = std::fs::File::create(&cache_dir)?;
        let mut patcher = patcher.lock().expect(concat!(file!(),":",line!()));
        patcher.rank_mirrors()?;
        let result = patcher.download_file_from_mirrors("/redists/UE3Redist.exe", file);
        drop(patcher);
        if let Err(error) = result {
          std::thread::spawn(move || {error_callback.call(None, &make_args!(format!("Failed to download UE3Redist: {}", error)), None).expect(concat!(file!(),":",line!()));});
          return Err::<(), Error>("Failed to download UE3Redist.".into());
        }

        //run installer of UE3Redist and quit this.
        match runas::Command::new(cache_dir.to_str().unwrap()).gui(true).status() {
          Ok(output) => {
            if output.success() {
              std::thread::spawn(move || {done.call(None, &make_args!(), None).expect(concat!(file!(),":",line!()));});
            } else {
              std::thread::spawn(move || {error_callback.call(None, &make_args!(format!("UE3Redist.exe exited in a crash: {}", output.code().expect(concat!(file!(),":",line!())))), None).expect(concat!(file!(),":",line!()));});
            }
          },
          Err(e) => {
            eprintln!("Failed to create child: {}", &e);
            std::thread::spawn(move || {error_callback.call(None, &make_args!(format!("Failed to open UE3Redist: {}", &e)), None).expect(concat!(file!(),":",line!()));});
          }
        };
        Ok::<(), Error>(())
      });
    }
  }

  /// Launcher updater
  fn update_launcher(&self, progress: Value) {
    let launcher_info = self.patcher.lock().expect(concat!(file!(),":",line!())).get_launcher_info().expect(concat!(file!(),":",line!()));
    if VERSION != launcher_info.version_name {
      let socket_addrs = launcher_info.patch_url.parse::<url::Url>().unwrap().socket_addrs(|| None).unwrap();
      let uri = launcher_info.patch_url.parse::<hyper::Uri>().unwrap();
      let good_hash = launcher_info.patch_hash.clone();
      drop(launcher_info);
      std::thread::spawn(move || {
        let mut rt = tokio::runtime::Builder::new().basic_scheduler().enable_time().enable_io().build().unwrap();
        let result = rt.enter(|| {
          rt.spawn(async move {
            //Connect tcp stream to a hostname:port
            let tls : tokio_tls::TlsConnector = native_tls::TlsConnector::new().unwrap().into();
            let resolver_service = ResolverService::new(socket_addrs.into());
            let mut http_connector : HttpConnector<ResolverService> = HttpConnector::new_with_resolver(resolver_service);
            http_connector.enforce_http(false);
            let https_connector : hyper_tls::HttpsConnector<HttpConnector<ResolverService>> = (http_connector, tls).into();
            let client = Client::builder().build::<_, hyper::Body>(https_connector);

            // Set up a request
            let req = hyper::Request::builder();
            let req = req.uri(uri).header("User-Agent", "sonny-launcher/1.0");
            let req = req.body(hyper::Body::empty()).expect(concat!(file!(),":",line!()));

            // Send request
            let res = client.request(req).await?;

            if res.status() == 200 || res.status() == 206 {
              // Initialize progress in front-end to be 0 up to maximum content_length
              let content_length : usize = res.headers().get("content-length").expect("Expected a content-length header, however none was found.").to_str().expect("Couldn't convert content-length value to str.").parse().expect("Couldn't parse content-length as a usize.");
              let progress_clone = progress.clone();
              std::thread::spawn(move || {
                progress.call(None, &make_args!(format!("[0, {}]", content_length)), None).expect(concat!(file!(),":",line!()));
              });

              // Set up vector where the stream will write into
              let mut download_contents = Vec::with_capacity(content_length);
              let mut downloaded = 0;
              let mut body = res.into_body();

              while !body.is_end_stream() {
                if let Some(chunk) = body.data().await {
                  let chunk = chunk?;
                  let chunk_size = chunk.len();
                  downloaded += chunk_size;
                  let progress_clone = progress_clone.clone();
                  if downloaded*100/content_length > (downloaded-chunk_size)*100/content_length {
                    std::thread::spawn(move || {
                      progress_clone.call(None, &make_args!(format!("[{},{}]", downloaded.to_string(), content_length.to_string())), None).expect(concat!(file!(),":",line!()));
                    });
                  }
                  download_contents.write_all(&chunk)?;
                }
              }
              drop(client);

              // check instructions hash
              if &good_hash != "" {
                let mut sha256 = Sha256::new();
                sha256.input(&download_contents);
                let hash = hex::encode_upper(sha256.result());
                if &hash != &good_hash {
                  panic!("The hashes don't match one another!");
                }
              }

              let download_contents = std::io::Cursor::new(download_contents);
              let mut output_path = std::env::current_exe().expect(concat!(file!(),":",line!()));
              output_path.pop();
              let target_dir = output_path.clone();
              output_path.pop();
              let working_dir = output_path.clone();
              output_path.push("launcher_update_extracted/");
              println!("{:?}", output_path);
              let mut self_update_executor = output_path.clone();

              //extract files
              unzip::Unzipper::new(download_contents, output_path).unzip().expect(concat!(file!(),":",line!()));

              //run updater program and quit this.
              self_update_executor.push("SelfUpdateExecutor.exe");
              let args = vec![format!("--pid={}",std::process::id()), format!("--target={}", target_dir.to_str().expect(concat!(file!(),":",line!())))];
              std::process::Command::new(self_update_executor)
                                          .current_dir(working_dir)
                                          .args(&args)
                                          .stdout(std::process::Stdio::piped())
                                          .stderr(std::process::Stdio::inherit())
                                          .spawn().expect(concat!(file!(),":",line!()));
              std::process::exit(0);
            }
            Err::<(), Error>("Launcher update: File not found.".into())
          })
        });
        let _ = rt.block_on(result).unwrap();
      });
    }
  }

  /// Fetch the text-resource at url with the specified headers.
  fn fetch_resource(&self, url: Value, mut headers_value: Value, callback: Value, context: Value) {
    let mut headers = HeaderMap::new();
    headers_value.isolate();
    for (key,value) in headers_value.items() {
      if let Ok(value) = HeaderValue::from_str(&value.as_string().expect("header value was empty.")) {
        let key = HeaderName::from_bytes(key.as_string().expect("Key value was empty.").as_bytes()).expect("Invalid Header Name");
        headers.insert(key, value);
      }
    }
    let url = url.as_string().expect("Couldn't parse url as string.").parse::<hyper::Uri>();
    std::thread::spawn(move || {
      let mut rt = tokio::runtime::Builder::new().basic_scheduler().enable_time().enable_io().build().unwrap();
      let result = rt.enter(|| {
        rt.spawn(async move {
          let url = url?;
          if let Some(host) = url.host() {
            let https = hyper_tls::HttpsConnector::new();
            let client = hyper::Client::builder().build::<_, hyper::Body>(https);
            let mut req = hyper::Request::builder();
            let headers_mut = req.headers_mut().unwrap();
            *headers_mut = headers;

            let req = req.uri(url.clone()).header("host", host).header("User-Agent", format!("RenX-Launcher ({})", VERSION));
            let req = req.body(hyper::Body::empty()).expect(concat!(file!(),":",line!()));
            let res = client.request(req).await?;

            let abort_in_error = res.status() != 200 && res.status() != 206;
            let body = hyper::body::to_bytes(res.into_body()).await?;
            std::thread::spawn(move || {
              if !abort_in_error {
                let text = ::std::str::from_utf8(&body).expect("Expected an utf-8 string");
                callback.call(Some(context), &make_args!(text), None).expect(concat!(file!(),":",line!()));
              } else {
                callback.call(Some(context), &make_args!(""), None).expect(concat!(file!(),":",line!()));
              }
            });
          }
          Ok::<(), Error>(())
        })
      });
      let _ = rt.block_on(result).unwrap();
    });
  }

  /// Fetch the image at url with specified headers
  fn fetch_image(&self, url: Value, mut headers_value: Value, callback: Value, context: Value) {
    let mut headers = HeaderMap::new();
    headers_value.isolate();
    for (key,value) in headers_value.items() {
      if let Ok(value) = HeaderValue::from_str(&value.as_string().expect("header value was empty.")) {
        let key = HeaderName::from_bytes(key.as_string().expect("Key value was empty.").as_bytes()).expect("Invalid Header Name");
        headers.insert(key, value);
      }
    }
    let url = url.as_string().expect("Couldn't parse url as string.").parse::<hyper::Uri>();
    std::thread::spawn(move || {
      let mut rt = tokio::runtime::Builder::new().basic_scheduler().enable_time().enable_io().build().unwrap();
      let result = rt.enter(|| {
        rt.spawn(async move {
          let url = url?;
          if let Some(host) = url.host() {
            let https = hyper_tls::HttpsConnector::new();
            let client = hyper::Client::builder().build::<_, hyper::Body>(https);
            let mut req = hyper::Request::builder();
            let headers_mut = req.headers_mut().unwrap();
            *headers_mut = headers;

            let req = req.uri(url.clone()).header("host", host).header("User-Agent", format!("RenX-Launcher ({})", VERSION));
            let req = req.body(hyper::Body::empty()).expect(concat!(file!(),":",line!()));
            let res = client.request(req).await?;

            let abort_in_error = res.status() != 200 && res.status() != 206;
            let body = hyper::body::to_bytes(res.into_body()).await?;
            std::thread::spawn(move || {
              if !abort_in_error {
                callback.call(Some(context), &make_args!(body.as_ref()), None).expect(concat!(file!(),":",line!()));
              } else {
                callback.call(Some(context), &make_args!(Value::null()), None).expect(concat!(file!(),":",line!()));
              }
            });
          }
          Ok::<(), Error>(())
        })
      });
      let _ = rt.block_on(result).unwrap();
    });
  }
}

impl sciter::EventHandler for Handler {
	dispatch_script_call! {
    fn check_update(Value, Value);
    fn install_redists(Value, Value);
    fn start_download(Value, Value, Value);
    fn remove_unversioned(Value, Value);

    fn get_playername();

    fn get_game_version();
    fn set_playername(Value);

    fn get_servers(Value);
    fn launch_game(Value, Value, Value); //Parameters: (Server IP+Port, onDone, onError);
    fn get_ping(Value, Value);

    fn get_setting(Value);
    fn set_setting(Value, Value);
    fn get_launcher_version();
    fn check_launcher_update(Value);
    fn update_launcher(Value);
    fn fetch_resource(Value,Value,Value,Value);
    fn fetch_image(Value,Value,Value,Value);
  }
}

struct UpdateResultHandler {
  update_result: String,
}

impl UpdateResultHandler {
  fn get_return_code(&self) -> String {
    self.update_result.clone()
  }
}

impl sciter::EventHandler for UpdateResultHandler {
	dispatch_script_call! {
		fn get_return_code();
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let instance = SingleInstance::new("RenegadeX-Launcher")?;
  //TODO: Create "Another instance is already running" window.
  assert!(instance.is_single());

  sciter::set_options(
    sciter::RuntimeOptions::DebugMode(true)
  ).expect(concat!(file!(),":",line!()));
  sciter::set_options(
    sciter::RuntimeOptions::ScriptFeatures(
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO as u8 |
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8 |
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SOCKET_IO as u8 | // Enables connecting to the inspector via Ctrl+Shift+I
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_EVAL as u8  // Enables execution of Eval inside of TI-Script
    )
  ).expect(concat!(file!(),":",line!())); 

  let mut config_directory = dirs::config_dir().unwrap();
  config_directory.push("Renegade X");
  config_directory.push("Renegade X Launcher.ini");

  let mut current_dir = std::env::current_exe()?;
  current_dir.pop();
  std::env::set_current_dir(&current_dir)?;
  const WEBIFY: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC.remove(b'/').remove(b'\\').remove(b':');
  let current_dir = percent_encoding::utf8_percent_encode(current_dir.to_str().unwrap(), WEBIFY).to_string();

  for argument in std::env::args() {
    if argument.starts_with("--patch-result=") {
      let mut frame = sciter::Window::new();
      frame.event_handler(UpdateResultHandler{update_result: argument[15..].to_string()});
      frame.load_file(&format!("file://{}/dom/self-update-result.htm", &current_dir));
      frame.run_app();
    }
  }

  let conf = match Ini::load_from_file(config_directory.to_str().unwrap()) {
    Ok(conf) => conf,
    Err(_e) => {
      let mut conf = Ini::new();
      conf.with_section(Some("RenX_Launcher"))
        .set("GameLocation", "../")
        .set("VersionUrl", "https://static.renegade-x.com/launcher_data/version/launcher.json")
        .set("PlayerName", "UnknownPlayer")
        .set("LauncherTheme", "dom")
        .set("LastNewsGUID", "")
        .set("64-bit-version", "true")
        .set("skipMovies", "false");
      let conf_arc = Arc::new(Mutex::new(conf.clone()));
      {
        let mut frame = sciter::Window::new();
        let patcher : Arc<Mutex<Downloader>> = Arc::new(Mutex::new(Downloader::new()));
        frame.event_handler(Handler{patcher: patcher.clone(), conf: conf_arc.clone()});
        frame.load_file(&format!("file://{}/dom/first-startup.htm", &current_dir));
        frame.run_app();
      }
      conf = match Arc::try_unwrap(conf_arc) {
        Ok(conf_mutex) => {
          conf_mutex.into_inner().expect(concat!(file!(),":",line!())).clone()
        },
        Err(_e) => {
          panic!(concat!(file!(),":",line!(),": No way to deal with this for now"));
        }
      };
      conf
    }
  };

  let section = conf.section(Some("RenX_Launcher".to_owned())).expect(concat!(file!(),":",line!()));
  let game_location = section.get("GameLocation").expect(concat!(file!(),":",line!()));
  let version_url = section.get("VersionUrl").expect(concat!(file!(),":",line!()));
  let launcher_theme = section.get("LauncherTheme").expect(concat!(file!(),":",line!()));

  let mut frame = sciter::Window::new();
  let mut downloader = Downloader::new();
  downloader.set_location(game_location.to_string());
  downloader.set_version_url(version_url.to_string());
  let patcher : Arc<Mutex<Downloader>> = Arc::new(Mutex::new(downloader));
  let conf_arc = Arc::new(Mutex::new(conf.clone()));
  frame.event_handler(Handler{patcher: patcher.clone(), conf: conf_arc});
  println!("{}",&format!("file://{}/{}/frontpage.htm", current_dir, launcher_theme));
  frame.load_file(&format!("file://{}/{}/frontpage.htm", current_dir, launcher_theme));
  frame.run_app();
  Ok(())
}
