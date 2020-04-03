#![windows_subsystem="console"]
#![warn(clippy::multiple_crate_versions)]

extern crate native_tls;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio;
extern crate tokio_tls;
extern crate tokio_reactor;
#[macro_use] extern crate futures;
#[macro_use] extern crate sciter;
extern crate renegadex_patcher;
extern crate ini;
extern crate single_instance;
extern crate socket2;
extern crate rand;

extern crate unzip;

use std::sync::{Arc,Mutex};

use sciter::Value;

use socket2::*;

use renegadex_patcher::{Downloader,Update, traits::Error};
use ini::Ini;
use single_instance::SingleInstance;
use std::io::Write;
use futures::Future;

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
      let update = &progress.lock().expect(concat!(file!(),":",line!())).update;
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
          patcher.retrieve_mirrors()?;
          update_available = patcher.update_available()?;
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
        use std::error::Error;
        eprintln!("{:#?}", err.description());
        std::thread::spawn(move || {error.call(None, &make_args!(err.description()), None).expect(concat!(file!(),":",line!()));});
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
        locked_patcher.poll_progress();
        result = locked_patcher.download();
      }
      match result {
        Ok(()) => {
          println!("Calling download done");
          std::thread::spawn(move || {callback_done.call(None, &make_args!(false,false), None).expect(concat!(file!(),":",line!()));});
        },
        Err(e) => {
          use std::error::Error;
          eprintln!("{:#?}", e.description());
          std::thread::spawn(move || {error.call(None, &make_args!(e.description()), None).expect(concat!(file!(),":",line!()));});
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
        result = locked_patcher.remove_unversioned();
      }
      match result {
        Ok(()) => {
          println!("Calling remove unversioned done");
          std::thread::spawn(move || {callback_done.call(None, &make_args!("validate"), None).expect(concat!(file!(),":",line!()));});
        },
        Err(e) => {
          use std::error::Error;
          eprintln!("Error in remove_unversioned(): {:#?}", e.description());
          std::thread::spawn(move || {error.call(None, &make_args!(e.description()), None).expect(concat!(file!(),":",line!()));});
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
    conf.write_to_file("RenegadeX-Launcher.ini").expect(concat!(file!(),":",line!()));
  }

  /// Get Server List as plain text
  fn get_servers(&self, callback: sciter::Value) {
    std::thread::spawn(move || {
      let url = "https://serverlist.renegade-x.com/servers.jsp?id=launcher".parse::<hyper::Uri>().expect(concat!(file!(),":",line!()));
      let https = hyper_tls::HttpsConnector::new(4).expect("TLS initialization failed");
      let client = hyper::Client::builder().build::<_, hyper::Body>(https);
      let mut req = hyper::Request::builder();
      req.uri(url.clone()).header("host", url.host().expect(concat!(file!(),":",line!()))).header("User-Agent", format!("RenX-Launcher ({})", VERSION));
      let req = req.body(hyper::Body::empty()).expect(concat!(file!(),":",line!()));
      let res = client.request(req).and_then(|res| {
        use hyper::rt::*;
        let abort_in_error = res.status() != 200 && res.status() != 206;
        res.into_body().concat2().and_then(move |body| {
          if !abort_in_error {
            std::thread::spawn(move || {
              let text : Value = ::std::str::from_utf8(&body).expect("Expected an utf-8 string").parse().expect(concat!(file!(),":",line!()));
              callback.call(None, &make_args!(text), None).expect(concat!(file!(),":",line!()));
            });
          }
          Ok(())
        })
      });
      tokio::runtime::current_thread::Runtime::new().expect(concat!(file!(),":",line!())).block_on(res).expect(concat!(file!(),":",line!()));
    });
  }

  /// Get ping of server
  fn get_ping(&self, server: sciter::Value, callback: sciter::Value) {
    std::thread::spawn(move || {
      use std::str::FromStr;
      let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4())).expect(concat!(file!(),":",line!(),": New socket"));
      let sock_addr = std::net::SocketAddr::from_str(&server.as_string().expect(concat!(file!(),":",line!()))).expect(concat!(file!(),":",line!())).into();
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
      socket.recv(&mut buf).expect(concat!(file!(),":",line!()));
      let elapsed = start_time.elapsed().as_millis() as i32;
      if buf[36..36+48] == code[16..] {
        //println!("{:#?}", &elapsed);
        std::thread::spawn(move || {callback.call(None, &make_args!(server, elapsed), None).expect(concat!(file!(),":",line!()));});
      } else {
        //println!("{:?}", &buf[36..36+48]);
        //println!("{:?}", &code[16..]);
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
        section.get("GameVersion").expect(concat!(file!(),":",line!())).clone()
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
      let mut args = vec![server.as_string().expect(concat!(file!(),":",line!())), format!("-ini:UDKGame:DefaultPlayer.Name={}", playername)];
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
            //eprintln!("{:#?}", output.unwrap_err().description());
            std::thread::spawn(move || {error.call(None, &make_args!(format!("The game exited in a crash: {}", output.code().expect(concat!(file!(),":",line!())))), None).expect(concat!(file!(),":",line!()));});
          }
        },
        Err(e) => {
          use std::error::Error;
          eprintln!("Failed to create child: {}", e.description());
          std::thread::spawn(move || {error.call(None, &make_args!(format!("Failed to open game: {}", e.description())), None).expect(concat!(file!(),":",line!()));});
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
    conf.write_to_file("RenegadeX-Launcher.ini").expect(concat!(file!(),":",line!()));
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

  /// Launcher updater
  fn update_launcher(&self, progress: Value) {
    let launcher_info = self.patcher.lock().expect(concat!(file!(),":",line!())).get_launcher_info().expect(concat!(file!(),":",line!()));
    if VERSION != launcher_info.version_name {
      std::thread::spawn(move || {
        //download file
        let future;
        let download_contents = Arc::new(Mutex::new(Vec::new()));
        let download_contents_clone = download_contents.clone();
        {
          let url = launcher_info.patch_url.parse::<hyper::Uri>().expect(concat!(file!(),":",line!()));
          let host_port = format!("{}:{}",url.host().expect(concat!(file!(),":",line!())),url.port_u16().unwrap_or(80_u16));
          let tcpstream = std::net::TcpStream::connect(host_port).expect(concat!(file!(),":",line!()));
          future = tokio::net::TcpStream::from_std(tcpstream, &tokio_reactor::Handle::default()).map(|tcp| {
            hyper::client::conn::handshake(tcp)
          }).expect(concat!(file!(),":",line!())).and_then(move |(mut client, conn)| {
            let mut req = hyper::Request::builder();
            req.uri(url.path()).header("host", url.host().expect(concat!(file!(),":",line!()))).header("User-Agent", "sonny-launcher/1.0");
            let req = req.body(hyper::Body::empty()).expect(concat!(file!(),":",line!()));
            let res = client.send_request(req).and_then(move |res| {
              use hyper::rt::*;
              let abort_in_error = res.status() != 200 && res.status() != 206;
              let content_length : usize = res.headers().get("content-length").expect(concat!(file!(),":",line!())).to_str().expect(concat!(file!(),":",line!())).parse().expect(concat!(file!(),":",line!()));
              let progress_clone = progress.clone();
              std::thread::spawn(move || {progress.call(None, &make_args!(format!("[0, {}]", content_length)), None).expect(concat!(file!(),":",line!()));});
              *download_contents_clone.lock().expect(concat!(file!(),":",line!())) = Vec::with_capacity(content_length);
              let mut downloaded = 0;
              res.into_body().for_each(move |chunk| {
                let chunk_size = chunk.len();
                if !abort_in_error {
                  downloaded += chunk_size;
                  let progress_clone = progress_clone.clone();
                  if downloaded*100/content_length > (downloaded-chunk_size)*100/content_length {
                    std::thread::spawn(move || {progress_clone.call(None, &make_args!(format!("[{},{}]", downloaded.to_string(), content_length.to_string())), None).expect(concat!(file!(),":",line!()));});
                  }
                  download_contents_clone.lock().expect(concat!(file!(),":",line!())).write_all(&chunk).map_err(|e| panic!("Writer encountered an error: {}", e))
                } else {
                  vec![].write_all(&chunk).map_err(|e| panic!("Writer encountered an error: {}", e))
                }
              })
            });
            // Put in an Option so poll_fn can return it later
            let mut conn = Some(conn);
            let until_upgrade = futures::future::poll_fn(move || {
              try_ready!(conn.as_mut().expect(concat!(file!(),":",line!())).poll_without_shutdown());
              Ok(futures::Async::Ready(conn.take().expect(concat!(file!(),":",line!()))))
            });
            res.join(until_upgrade)
          }).and_then(move |(result, client)| {
            drop(client);
            Ok(result)
          });
        }
        tokio::runtime::current_thread::Runtime::new().expect(concat!(file!(),":",line!())).block_on(future).expect(concat!(file!(),":",line!()));

        //extract files
        let download_contents = std::io::Cursor::new(Arc::try_unwrap(download_contents).expect(concat!(file!(),":",line!())).into_inner().expect(concat!(file!(),":",line!())));
        let mut output_path = std::env::current_exe().expect(concat!(file!(),":",line!()));
        output_path.pop();
        let target_dir = output_path.clone();
        output_path.pop();
        let working_dir = output_path.clone();
        
        output_path.push("launcher_update_extracted/");
        println!("{:?}", output_path);
        let mut self_update_executor = output_path.clone();
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
      });
    }
  }

  /// Fetch the text-resource at url with the specified headers.
  fn fetch_resource(&self, url: Value, mut headers_value: Value, callback: Value, context: Value) {
    std::thread::spawn(move || {
      let url = url.as_string().expect(concat!(file!(),":",line!())).parse::<hyper::Uri>().expect(concat!(file!(),":",line!()));
      let https = hyper_tls::HttpsConnector::new(4).expect("TLS initialization failed");
      let client = hyper::Client::builder().build::<_, hyper::Body>(https);
      let mut req = hyper::Request::builder();
      req.uri(url.clone()).header("host", url.host().expect(concat!(file!(),":",line!()))).header("User-Agent", format!("RenX-Launcher ({})", VERSION));
      headers_value.isolate();
      for (key,value) in headers_value.items() {
        req.header(key.as_string().expect(concat!(file!(),":",line!())).as_bytes(), value.as_string().expect(concat!(file!(),":",line!())));
      }
      let req = req.body(hyper::Body::empty()).expect(concat!(file!(),":",line!()));
      let res = client.request(req).and_then(|res| {
        use hyper::rt::*;
        let abort_in_error = res.status() != 200 && res.status() != 206;
        res.into_body().concat2().and_then(move |body| {
          std::thread::spawn(move || {
            if !abort_in_error {
              let text = ::std::str::from_utf8(&body).expect("Expected an utf-8 string");
              callback.call(Some(context), &make_args!(text), None).expect(concat!(file!(),":",line!()));
            } else {
              callback.call(Some(context), &make_args!(""), None).expect(concat!(file!(),":",line!()));
            }
          });
          Ok(())
        })
      });
      tokio::runtime::current_thread::Runtime::new().expect(concat!(file!(),":",line!())).block_on(res).expect(concat!(file!(),":",line!()));
    });
  }

  /// Fetch the image at url with specified headers
  fn fetch_image(&self, url: Value, mut headers_value: Value, callback: Value, context: Value) {
    std::thread::spawn(move || {
      let url = url.as_string().expect(concat!(file!(),":",line!())).parse::<hyper::Uri>().expect(concat!(file!(),":",line!()));
      let https = hyper_tls::HttpsConnector::new(4).expect("TLS initialization failed");
      let client = hyper::Client::builder().build::<_, hyper::Body>(https);
      let mut req = hyper::Request::builder();
      req.uri(url.clone()).header("host", url.host().expect(concat!(file!(),":",line!()))).header("User-Agent", format!("RenX-Launcher ({})", VERSION));
      headers_value.isolate();
      for (key,value) in headers_value.items() {
        req.header(key.as_string().expect(concat!(file!(),":",line!())).as_bytes(), value.as_string().expect(concat!(file!(),":",line!())));
      }
      let req = req.body(hyper::Body::empty()).expect(concat!(file!(),":",line!()));
      let res = client.request(req).and_then(|res| {
        use hyper::rt::*;
        let abort_in_error = res.status() != 200 && res.status() != 206;
        res.into_body().concat2().and_then(move |body| {
          std::thread::spawn(move || {
            if !abort_in_error {
              callback.call(Some(context), &make_args!(body.as_ref()), None).expect(concat!(file!(),":",line!()));
            } else {
              callback.call(Some(context), &make_args!(Value::null()), None).expect(concat!(file!(),":",line!()));
            }
          });
          Ok(())
        })
      });
      tokio::runtime::current_thread::Runtime::new().expect(concat!(file!(),":",line!())).block_on(res).expect(concat!(file!(),":",line!()));
    });
  }
}

impl sciter::EventHandler for Handler {
	dispatch_script_call! {
		fn check_update(Value, Value);
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

fn main() {
  let instance = SingleInstance::new("RenegadeX-Launcher").expect(concat!(file!(),":",line!()));
  //TODO: Create "Another instance is already running" window.
  assert!(instance.is_single());

  let conf = match Ini::load_from_file("RenegadeX-Launcher.ini") {
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
        sciter::set_options(
          sciter::RuntimeOptions::ScriptFeatures(
            sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO as u8 |
            sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8 |
            sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SOCKET_IO as u8 | // Enables connecting to the inspector via Ctrl+Shift+I
            sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_EVAL as u8  // Enables execution of Eval inside of TI-Script
          )
        ).expect(concat!(file!(),":",line!())); 
        let mut frame = sciter::Window::new();
        let patcher : Arc<Mutex<Downloader>> = Arc::new(Mutex::new(Downloader::new()));
        frame.event_handler(Handler{patcher: patcher.clone(), conf: conf_arc.clone()});
        let mut current_path = std::env::current_exe().expect(concat!(file!(),":",line!()));
        current_path.pop();
        frame.load_file(&format!("file://{}/dom/first-startup.htm", current_path.to_str().expect(concat!(file!(),":",line!()))));
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

  let mut current_path = std::env::current_exe().expect(concat!(file!(),":",line!()));
  current_path.pop();
  sciter::set_options(
    sciter::RuntimeOptions::ScriptFeatures(
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO as u8 |
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8 |
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SOCKET_IO as u8 | // Enables connecting to the inspector via Ctrl+Shift+I
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_EVAL as u8  // Enables execution of Eval inside of TI-Script
    )
  ).expect(concat!(file!(),":",line!())); 
  let mut frame = sciter::Window::new();
  let mut downloader = Downloader::new();
  downloader.set_location(game_location.to_string());
  downloader.set_version_url(version_url.to_string());
  let patcher : Arc<Mutex<Downloader>> = Arc::new(Mutex::new(downloader));
  let conf_arc = Arc::new(Mutex::new(conf.clone()));
  frame.event_handler(Handler{patcher: patcher.clone(), conf: conf_arc});
  frame.load_file(&format!("file://{}/{}/frontpage.htm", current_path.to_str().expect(concat!(file!(),":",line!())), launcher_theme));
  frame.run_app();
}
