#![windows_subsystem="windows"]
#![warn(clippy::multiple_crate_versions)]
extern crate tokio;
#[macro_use] extern crate sciter;
extern crate renegadex_patcher;
extern crate ini;
extern crate single_instance;
extern crate socket2;
extern crate rand;
extern crate unzip;
extern crate dirs;
extern crate runas;
extern crate sha2;
extern crate hex;
extern crate log;
extern crate download_async;
extern crate async_trait;

mod configuration;

use flexi_logger::{Age, Criterion, Cleanup, Logger, Naming};
use log::*;
use renegadex_patcher::{Downloader,Update, traits::Error};
use sciter::Value;
use sha2::{Sha256, Digest};
use single_instance::SingleInstance;
use socket2::*;
use async_trait::async_trait;

use std::io::Write;
use std::net::ToSocketAddrs;
use std::sync::{Arc,Mutex};

pub struct Progress {

}

#[async_trait]
impl download_async::Progress for Progress {
  async fn get_file_size(&self) -> usize {
    64
  }

  async fn get_progess(&self) -> usize {
    64
  }

  async fn set_file_size(&mut self, size: usize) {

  }

  async fn add_to_progress(&mut self, amount: usize) {

  }

  async fn remove_from_progress(&mut self, bytes: usize) {

  }
}

pub struct ValueProgress {
  progress: std::sync::Arc<std::sync::Mutex<Value>>,
  file_size: usize,
  downloaded: usize,
}

impl ValueProgress {
  pub fn new(value: Value) -> Self {
    Self {
      progress: std::sync::Arc::new(std::sync::Mutex::new(value)),
      file_size: 0,
      downloaded: 0
    }
  }
}

#[async_trait]
impl download_async::Progress for ValueProgress {
  async fn get_file_size(&self) -> usize {
    64
  }

  async fn get_progess(&self) -> usize {
    64
  }

  async fn set_file_size(&mut self, size: usize) {
    self.file_size = size;

    let file_size = self.file_size;
    let downloaded = 0;
    let progress = self.progress.lock().unwrap().clone();
    std::thread::spawn(move || {
      progress.call(None, &make_args!(format!("[{}, {}]", downloaded, file_size)), None).unexpected(concat!(file!(),":",line!()));
    });
  }

  async fn add_to_progress(&mut self, amount: usize) {
    self.downloaded += amount;
    let file_size = self.file_size;
    let downloaded = self.downloaded;
    let progress = self.progress.lock().unwrap().clone();
    std::thread::spawn(move || {
      progress.call(None, &make_args!(format!("[{}, {}]", downloaded, file_size)), None).unexpected(concat!(file!(),":",line!()));
    });
  }

  async fn remove_from_progress(&mut self, bytes: usize) {

  }
}

pub trait ExpectUnwrap<T> :  {
  fn unexpected(self, msg: &str) -> T;
}

impl<T, E: std::fmt::Debug> ExpectUnwrap<T> for Result<T, E> {
  #[inline]
  fn unexpected(self, msg: &str) -> T {
    match self {
      Ok(val) => val,
      Err(e) => unwrap_failed(msg, &e),
    }
  }
}

impl<T> ExpectUnwrap<T> for Option<T> {
  #[inline]
  fn unexpected(self, msg: &str) -> T {
    match self {
      Some(val) => val,
      None => expect_failed(msg),
    }
  }
}

#[inline(never)]
#[cold]
fn expect_failed(msg: &str) -> ! {
  error!("{}", msg);
  log::logger().flush();
  panic!("{}", msg)
}

#[inline(never)]
#[cold]
fn unwrap_failed(msg: &str, error: &dyn std::fmt::Debug) -> ! {
  error!("{}: {:?}", msg, error);
  log::logger().flush();
  panic!("{}: {:?}", msg, error)
}

/// The current launcher's version
static VERSION: &str = env!("CARGO_PKG_VERSION");

/// Structure for Sciter event handling.
struct Handler {
  /// The reference to the back-end library which is responsible for downloading and updating the game.
  patcher: Arc<Mutex<Downloader>>,
  /// The configuration file for the launcher.
  configuration: configuration::Configuration
}

impl Handler {
  /// Check if there are game updates available, makes use of caching.
  fn check_update(&self, done: sciter::Value, error: sciter::Value) {
    {
      info!("Checking for an update!");

      let progress = self.patcher.clone().lock().unexpected(concat!(file!(),":",line!())).get_progress();
      let update = &progress.lock().unexpected(concat!(file!(),":",line!())).update.clone();
      match update {
        Update::UpToDate => {
          std::thread::spawn(move || {done.call(None, &make_args!("up_to_date"), None).unexpected(concat!(file!(),":",line!()));});
          return;
        },
        Update::Full => {
          std::thread::spawn(move || {done.call(None, &make_args!("full"), None).unexpected(concat!(file!(),":",line!()));});
          return;
        },
        Update::Resume => {
          std::thread::spawn(move || {done.call(None, &make_args!("resume"), None).unexpected(concat!(file!(),":",line!()));});
          return;
        },
        Update::Delta => {
          std::thread::spawn(move || {done.call(None, &make_args!("update"), None).unexpected(concat!(file!(),":",line!()));});
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
          let mut patcher = patcher.lock().unexpected(concat!(file!(),":",line!()));
          patcher.retrieve_mirrors().unexpected(concat!(file!(),":",line!()));
          update_available = patcher.update_available().unexpected(concat!(file!(),":",line!()));
        }
        match update_available {
          Update::UpToDate => {
            std::thread::spawn(move || {done.call(None, &make_args!("up_to_date"), None).unexpected(concat!(file!(),":",line!()));});
          },
          Update::Full => {
            std::thread::spawn(move || {done.call(None, &make_args!("full"), None).unexpected(concat!(file!(),":",line!()));});
          },
          Update::Resume => {
            std::thread::spawn(move || {done.call(None, &make_args!("resume"), None).unexpected(concat!(file!(),":",line!()));});
          },
          Update::Delta => {
            std::thread::spawn(move || {done.call(None, &make_args!("patch"), None).unexpected(concat!(file!(),":",line!()));});
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
        std::thread::spawn(move || {error.call(None, &make_args!(err.to_string()), None).unexpected(concat!(file!(),":",line!()));});
      }
    });
  }

  /// Starts the downloading of the update/game
  fn start_download(&self, callback: sciter::Value, callback_done: sciter::Value, error: sciter::Value) {
    info!("Starting game download!");

    let progress = self.patcher.clone().lock().unexpected(concat!(file!(),":",line!())).get_progress();
		std::thread::spawn(move || {
      let mut not_finished = true;
      let mut last_download_size : u64 = 0;
      while not_finished {
        std::thread::sleep(std::time::Duration::from_millis(500));
        {
          let progress_locked = progress.lock().unexpected(concat!(file!(),":",line!()));

          let sizes = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
          let bytes = ((progress_locked.download_size.0 - last_download_size) * 2) as f64;
          let base = bytes.log(1024_f64).floor() as usize;
          let speed = format!("{:.2} {}/s", bytes / 1024_u64.pow(base as u32) as f64, sizes[base]);

          let json = format!(
            "{{\"hash\": [{},{}],\"download\": [{}.0,{}.0],\"patch\": [{},{}],\"download_speed\": \"{}\"}}",
            progress_locked.hashes_checked.0,
            progress_locked.hashes_checked.1,
            progress_locked.download_size.0,
            progress_locked.download_size.1,
            progress_locked.patch_files.0,
            progress_locked.patch_files.1,
            speed
          );
          let me : Value = json.parse().unexpected(concat!(file!(),":",line!()));
          last_download_size = progress_locked.download_size.0;
          not_finished = !progress_locked.finished_patching;
          drop(progress_locked);
          let callback_clone = callback.clone();
          std::thread::spawn(move || {callback_clone.call(None, &make_args!(me), None).unexpected(concat!(file!(),":",line!()));});
        }
      }
		});
    let patcher = self.patcher.clone();
    std::thread::spawn(move || {
      let result : Result<(), renegadex_patcher::traits::Error>;
      {
        let mut locked_patcher = patcher.lock().unexpected(concat!(file!(),":",line!()));
        locked_patcher.rank_mirrors().unexpected(concat!(file!(),":",line!()));
        locked_patcher.poll_progress();
        result = locked_patcher.download();
      }
      match result {
        Ok(()) => {
          info!("Calling download done");
          std::thread::spawn(move || {callback_done.call(None, &make_args!(false,false), None).unexpected(concat!(file!(),":",line!()));});
        },
        Err(e) => {
          error!("{:#?}", &e);
          std::thread::spawn(move || {error.call(None, &make_args!(e.to_string()), None).unexpected(concat!(file!(),":",line!()));});
        }
      };
    });
  }

  /// Removes files inside of the subdirectories that are not part of the instructions.json
  fn remove_unversioned(&self, callback_done: sciter::Value, error: sciter::Value) {
    info!("Removing unused!");

    let patcher = self.patcher.clone();
    std::thread::spawn(move || {
      let result : Result<(), renegadex_patcher::traits::Error>;
      {
        let mut locked_patcher = patcher.lock().unexpected(concat!(file!(),":",line!()));
        locked_patcher.rank_mirrors().unexpected(concat!(file!(),":",line!()));
        result = locked_patcher.remove_unversioned();
      }
      match result {
        Ok(()) => {
          info!("Calling remove unversioned done");
          std::thread::spawn(move || {callback_done.call(None, &make_args!("validate"), None).unexpected(concat!(file!(),":",line!()));});
        },
        Err(e) => {
          error!("Error in remove_unversioned(): {:#?}", &e);
          std::thread::spawn(move || {error.call(None, &make_args!(e.to_string()), None).unexpected(concat!(file!(),":",line!()));});
        }
      };
    });
  }

  fn get_video_location(&self, map_name: sciter::Value) -> String {
    self.configuration.get_video_location(map_name.to_string())
  }

  /// Retrieve the playername
  fn get_playername(&self) -> String {
    info!("Requested playername!");
    self.configuration.get_playername()
  }

  /// Set the playername
  fn set_playername(&self, username: sciter::Value) {
    info!("Setting playername!");
    self.configuration.set_playername(&username.as_string().expect(""))
  }

  /// Get Server List as plain text
  fn get_servers(&self, callback: sciter::Value) {
    info!("Getting Servers!");

    std::thread::spawn(move || {
      let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unexpected(concat!(file!(),":",line!()));
      let _guard = rt.enter();
      let result = rt.spawn(async move {
        let url = "https://serverlist.renegade-x.com/servers.jsp?id=launcher".parse::<download_async::http::Uri>()?;
        let req = download_async::http::Request::builder();
        if let Some(host) = url.host() {
          let req = req.uri(url.clone()).header("host", host).header("User-Agent", format!("RenX-Launcher ({})", VERSION));
          let req = req.body(download_async::Body::empty())?;

          let mut buffer = vec![];
          let mut progress : Option<&mut Progress> = None;

          let response = download_async::download(req, &mut buffer, true, &mut progress, None).await;
          if response.is_ok() {
            std::thread::spawn(move || {
              let text : Value = std::str::from_utf8(&buffer).unexpected("Expected an utf-8 string").parse().unexpected(concat!(file!(),":",line!()));
              callback.call(None, &make_args!(text), None).unexpected(concat!(file!(),":",line!()));
            });
            Ok::<(), Error>(())
          } else {
            Err::<(), Error>(Error::new("".to_owned()))
          }
        } else {
          Err::<(), Error>(Error::new("".to_owned()))
        }
      });
      let _ = rt.block_on(result).unexpected(concat!(file!(),":",line!()));
    });
  }

  /// Get ping of server
  fn get_ping(&self, server: sciter::Value, callback: sciter::Value) {
    std::thread::spawn(move || {
      let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4())).unexpected(concat!(file!(),":",line!(),": New socket"));
      let server_string = server.as_string().unexpected(&format!("Couldn't cast server \"{:?}\" to string", &server));
      let mut server_socket = server_string.to_socket_addrs().unexpected(&format!("Couldn't unwrap socket address of server \"{}\"", &server_string));
      let sock_addr = server_socket.next().unexpected(&format!("No Sockets found for DNS name \"{}\"", &server_string)).into();
      let start_time = std::time::Instant::now();
      socket.connect_timeout(&sock_addr, std::time::Duration::from_millis(500)).unexpected(concat!(file!(),":",line!()));
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
      socket.send(&code).unexpected(concat!(file!(),":",line!()));
      let mut buf : [u8; 100] = [0; 100];
      socket.set_read_timeout(Some(std::time::Duration::from_millis(500))).unexpected(concat!(file!(),":",line!()));
      let result = socket.recv(&mut buf);
      let elapsed = start_time.elapsed().as_millis() as i32;
      if result.is_ok() && buf[36..36+48] == code[16..] {
        std::thread::spawn(move || {callback.call(None, &make_args!(server, elapsed), None).unexpected(concat!(file!(),":",line!()));});
      }
    });
  }

  /// Get the installed game's version
  fn get_game_version(&self) -> String {
    info!("Getting game version!");
    self.configuration.get_game_version()
  }

  /// Launch the game, if server variable it's value is "", then the game will be launched to the menu.
  fn launch_game(&self, server: Value, done: Value, error: Value) {
    info!("Launching game!");
    let game_location = self.configuration.get_game_location();
    let launch_info =  self.configuration.get_launch_info();

    std::thread::spawn(move || {
      let server = server.as_string().unexpected(concat!(file!(),":",line!()));
      let mut args = vec![];
      match server.as_str() {
        "" => {},
        _ => args.push(server)
      };
      args.push(format!("-ini:UDKGame:DefaultPlayer.Name={}", &launch_info.player_name));
      if launch_info.startup_movie_disabled {
        args.push("-nomoviestartup".to_string());
      }
      args.push("-UseAllAvailableCores".to_string());

      match std::process::Command::new(format!("{}/Binaries/Win{}/UDK.exe", game_location, launch_info.bit_version))
                                     .args(&args)	
                                     .stdout(std::process::Stdio::piped())
                                     .stderr(std::process::Stdio::inherit())
                                     .spawn() {
        Ok(mut child) => {
          let output = child.wait().unexpected("Failed to wait on game-instance to finish");
          if output.success() {
            std::thread::spawn(move || {done.call(None, &make_args!(), None).unexpected(concat!(file!(),":",line!()));});
          } else {
            //eprintln!("{:#?}", output.unwrap_err());
            std::thread::spawn(move || {error.call(None, &make_args!(format!("The game exited in a crash: {}", output.code().unexpected(concat!(file!(),":",line!())))), None).unexpected(concat!(file!(),":",line!()));});
          }
        },
        Err(e) => {
          error!("Failed to open game: {}", &e);
          std::thread::spawn(move || {error.call(None, &make_args!(format!("Failed to open game: {}", &e)), None).unexpected(concat!(file!(),":",line!()));});
        }
      };
    });
  }

  /// Gets the setting from the launchers configuration file.
  fn get_setting(&self, setting: sciter::Value) -> String {
    info!("Getting settings!");
    self.configuration.get_global_setting(&setting.as_string().expect(""))
  }

  /// Sets the setting in the launchers configuration file.
  fn set_setting(&self, setting: sciter::Value, value: sciter::Value) {
    info!("Setting settings!");
    self.configuration.set_global_setting(&setting.as_string().expect(""), &value.as_string().expect(""))
  }

  /// Get the current launcher version
  fn get_launcher_version(&self) -> &str {
    VERSION
  }

  /// Checks if the launcher is up to date
  fn check_launcher_update(&self, callback: Value) {
    info!("Checking for launcher update!");

    let launcher_info_option = self.patcher.lock().unexpected(concat!(file!(),":",line!())).get_launcher_info();
    if let Some(launcher_info) = launcher_info_option {
      if VERSION != launcher_info.version_name && !launcher_info.prompted {
        std::thread::spawn(move || {callback.call(None, &make_args!(launcher_info.version_name), None).unexpected(concat!(file!(),":",line!()));});
      } else {
        std::thread::spawn(move || {callback.call(None, &make_args!(Value::null()), None).unexpected(concat!(file!(),":",line!()));});
      }
    } else {
      let patcher = self.patcher.clone();
      std::thread::spawn(move || {
        let mut patcher = patcher.lock().unexpected(concat!(file!(),":",line!()));
        patcher.retrieve_mirrors().unexpected(concat!(file!(),":",line!()));
        let launcher_info_option = patcher.get_launcher_info();
        drop(patcher);
        if let Some(launcher_info) = launcher_info_option {
          if VERSION != launcher_info.version_name && !launcher_info.prompted {
            std::thread::spawn(move || {callback.call(None, &make_args!(launcher_info.version_name), None).unexpected(concat!(file!(),":",line!()));});
          } else {
            std::thread::spawn(move || {callback.call(None, &make_args!(Value::null()), None).unexpected(concat!(file!(),":",line!()));});
          }
        }
      });
    }
  }

  fn install_redists(&self, done: Value, error_callback: Value) {
    info!("Installing redistributables!");

    if let Some(mut cache_dir) = dirs::cache_dir() {
      let patcher = self.patcher.clone();
      // Spawn thread, to not block the main process.
      std::thread::spawn(move || {
        cache_dir.set_file_name("UE3Redist.exe");
        let file = std::fs::File::create(&cache_dir)?;
        let mut patcher = patcher.lock().unexpected(concat!(file!(),":",line!()));
        patcher.rank_mirrors()?;
        let result = patcher.download_file_from_mirrors("/redists/UE3Redist.exe", file);
        drop(patcher);
        if let Err(error) = result {
          std::thread::spawn(move || {error_callback.call(None, &make_args!(format!("Failed to download UE3Redist: {}", error)), None).unexpected(concat!(file!(),":",line!()));});
          return Err::<(), Error>("Failed to download UE3Redist.".into());
        }

        //run installer of UE3Redist and quit this.
        match runas::Command::new(cache_dir.to_str().unexpected(concat!(file!(),":",line!()))).gui(true).spawn().unexpected("Couldn't spawn UE3Redist.").wait() {
          Ok(output) => {
            if output.success() {
              std::thread::spawn(move || {done.call(None, &make_args!(), None).unexpected(concat!(file!(),":",line!()));});
            } else {
              std::thread::spawn(move || {error_callback.call(None, &make_args!(format!("UE3Redist.exe exited in a crash: {}", output.code().unexpected(concat!(file!(),":",line!())))), None).unexpected(concat!(file!(),":",line!()));});
            }
          },
          Err(e) => {
            error!("Failed to open UE3Redist: {}", &e);
            std::thread::spawn(move || {error_callback.call(None, &make_args!(format!("Failed to open UE3Redist: {}", &e)), None).unexpected(concat!(file!(),":",line!()));});
          }
        };
        Ok::<(), Error>(())
      });
    }
  }

  /// Launcher updater
  fn update_launcher(&self, progress: Value) {
    info!("Updating launcher!");

    let launcher_info = self.patcher.lock().unexpected(concat!(file!(),":",line!())).get_launcher_info().unexpected(concat!(file!(),":",line!()));
    if VERSION != launcher_info.version_name {
      let socket_addrs = launcher_info.patch_url.parse::<url::Url>().unexpected(concat!(file!(),":",line!())).socket_addrs(|| None).unexpected(concat!(file!(),":",line!()));
      let uri = launcher_info.patch_url.parse::<download_async::http::Uri>().unexpected(concat!(file!(),":",line!()));
      let good_hash = launcher_info.patch_hash.clone();
      drop(launcher_info);
      std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unexpected(concat!(file!(),":",line!()));
        let _guard = rt.enter();
        let result = rt.spawn(async move {
          // Set up a request
          let req = download_async::http::Request::builder();
          let req = req.uri(uri).header("User-Agent", "sonny-launcher/1.0");
          let req = req.body(download_async::Body::empty()).unexpected(concat!(file!(),":",line!()));

          let mut value_progress = ValueProgress::new(progress.clone());
          let mut progress : Option<&mut ValueProgress> = Some(&mut value_progress);
          let mut buffer = vec![];
          let res = download_async::download(req, &mut buffer, false, &mut progress, Some(socket_addrs.into())).await;

          if res.is_ok() {

            // check instructions hash
            if &good_hash != "" {
              let mut sha256 = Sha256::new();
              sha256.write(&buffer)?;
              let hash = hex::encode_upper(sha256.finalize());
              if &hash != &good_hash {
                error!("The hashes don't match one another!");
                log::logger().flush();
                panic!("The hashes don't match one another!");
              }
            }

            let download_contents = std::io::Cursor::new(buffer);
            let mut output_path = std::env::current_exe().unexpected(concat!(file!(),":",line!()));
            output_path.pop();
            let target_dir = output_path.clone();
            output_path.pop();
            let working_dir = output_path.clone();
            output_path.push("launcher_update_extracted/");
            info!("Extracting launcher update to: {:?}", output_path);
            let mut self_update_executor = output_path.clone();

            //extract files
            let result = unzip::Unzipper::new(download_contents, output_path).unzip().unexpected(concat!(file!(),":",line!()));
            info!("{:#?}", result);
            
            //run updater program and quit this.
            self_update_executor.push("SelfUpdateExecutor.exe");
            let args = vec![format!("--pid={}",std::process::id()), format!("--target={}", target_dir.to_str().unexpected(concat!(file!(),":",line!())))];
            std::process::Command::new(self_update_executor)
                                        .current_dir(working_dir)
                                        .args(&args)
                                        .stdout(std::process::Stdio::piped())
                                        .stderr(std::process::Stdio::inherit())
                                        .spawn().unexpected(concat!(file!(),":",line!()));
            std::process::exit(0);
          }
          Err::<(), Error>("Launcher update: File not found.".into())
        });
        let _ = rt.block_on(result).unexpected(concat!(file!(),":",line!()));
      });
    }
  }

  /// Fetch the text-resource at url with the specified headers.
  fn fetch_resource(&self, url: Value, mut headers_value: Value, callback: Value, context: Value) {
    info!("Fetching resource!");

    let mut headers = download_async::http::HeaderMap::new();
    headers_value.isolate();
    for (key,value) in headers_value.items() {
      if let Ok(value) = download_async::http::HeaderValue::from_str(&value.as_string().unexpected("header value was empty.")) {
        let key = download_async::http::header::HeaderName::from_bytes(key.as_string().unexpected("Key value was empty.").as_bytes()).unexpected("Invalid Header Name");
        headers.insert(key, value);
      }
    }
    let url = url.as_string().unexpected("Couldn't parse url as string.").parse::<download_async::http::Uri>();
    std::thread::spawn(move || {
      let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unexpected(concat!(file!(),":",line!()));
      let _guard = rt.enter();
      let result = rt.spawn(async move {
        let url = url?;
        if let Some(host) = url.host() {
          let mut progress : Option<&mut Progress> = None;
          let sockets = None;
          let mut buffer = vec![];

          let mut req = download_async::http::Request::builder();
          let headers_mut = req.headers_mut().unexpected(concat!(file!(),":",line!()));
          *headers_mut = headers;
          let req = req.uri(url.clone()).header("host", host).header("User-Agent", format!("RenX-Launcher ({})", VERSION));
          let req = req.body(download_async::Body::empty()).unexpected(concat!(file!(),":",line!()));

          let result = download_async::download(req, &mut buffer, false, &mut progress, sockets).await;
          if result.is_ok() {
            std::thread::spawn(move || {
              let text = ::std::str::from_utf8(&buffer).unexpected("Expected an utf-8 string");
              callback.call(Some(context), &make_args!(text), None).unexpected(concat!(file!(),":",line!()));
            });
            Ok::<(), Error>(())
          } else {
            Err::<(), Error>(Error::new(format!("Download of {} failed, see: {}", url, result.unwrap_err())))
          }
        } else {
          Err::<(), Error>(Error::new("Not a valid url.".to_owned()))
        }
      });
      let _ = rt.block_on(result).unexpected(concat!(file!(),":",line!()));
    });
  }

  /// Fetch the image at url with specified headers
  fn fetch_image(&self, url: Value, mut headers_value: Value, callback: Value, context: Value) {
    let mut headers = download_async::http::HeaderMap::new();
    headers_value.isolate();
    for (key,value) in headers_value.items() {
      if let Ok(value) = download_async::http::HeaderValue::from_str(&value.as_string().unexpected("header value was empty.")) {
        let key = download_async::http::header::HeaderName::from_bytes(key.as_string().unexpected("Key value was empty.").as_bytes()).unexpected("Invalid Header Name");
        headers.insert(key, value);
      }
    }
    let url = url.as_string().unexpected("Couldn't parse url as string.").parse::<download_async::http::Uri>();
    std::thread::spawn(move || {
      let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unexpected(concat!(file!(),":",line!()));
      let _guard = rt.enter();
      let result = rt.spawn(async move {
        let url = url?;
        if let Some(host) = url.host() {
          let mut progress : Option<&mut Progress> = None;
          let sockets = None;
          let mut buffer = vec![];

          let mut req = download_async::http::Request::builder();
          let headers_mut = req.headers_mut().unexpected(concat!(file!(),":",line!()));
          *headers_mut = headers;
          let req = req.uri(url.clone()).header("host", host).header("User-Agent", format!("RenX-Launcher ({})", VERSION));
          let req = req.body(download_async::Body::empty()).unexpected(concat!(file!(),":",line!()));

          let result = download_async::download(req, &mut buffer, false, &mut progress, sockets).await;
          if result.is_ok() {
            std::thread::spawn(move || {
              callback.call(Some(context), &make_args!(buffer.as_slice()), None).unexpected(concat!(file!(),":",line!()));
            });
            Ok::<(), Error>(())
          } else {
            Err::<(), Error>(Error::new(format!("Download of {} failed, see: {}", url, result.unwrap_err())))
          }
        } else {
          Err::<(), Error>(Error::new("Not a valid url.".to_owned()))
        }
      });
      let _ = rt.block_on(result).unexpected(concat!(file!(),":",line!()));
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
    fn get_video_location(Value);
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

  let configuration = configuration::Configuration::load_or_default();
  let log_directory = configuration.get_log_directory();

  Logger::with_env_or_str("info")
    .format(flexi_logger::opt_format)
    .directory(&log_directory)
    .rotate(Criterion::Age(Age::Day), Naming::Numbers, Cleanup::KeepLogFiles(5))
    .print_message()
    .log_to_file()
    .start()
    .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

  info!("Starting RenegadeX Launcher version {}", &VERSION);

  sciter::set_options(
    sciter::RuntimeOptions::DebugMode(true)
  ).unexpected(concat!(file!(),":",line!()));
  sciter::set_options(
    sciter::RuntimeOptions::ScriptFeatures(
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO as u8 |
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8 |
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SOCKET_IO as u8 | // Enables connecting to the inspector via Ctrl+Shift+I
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_EVAL as u8  // Enables execution of Eval inside of TI-Script
    )
  ).unexpected(concat!(file!(),":",line!()));


  let mut current_dir = std::env::current_exe()?;
  current_dir.pop();
  info!("Working in directory: {}", &current_dir.to_string_lossy());
  std::env::set_current_dir(&current_dir)?;
  const WEBIFY: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC.remove(b'/').remove(b'\\').remove(b':');
  let current_dir = percent_encoding::utf8_percent_encode(current_dir.to_str().unexpected(concat!(file!(),":",line!())), WEBIFY).to_string();

  for argument in std::env::args() {
    if argument.starts_with("--patch-result=") {
      info!("Update result: {}", &argument[15..].to_string());
      let mut frame = sciter::Window::new();
      frame.event_handler(UpdateResultHandler{update_result: argument[15..].to_string()});
      frame.load_file(&format!("file://{}/dom/self-update-result.htm", &current_dir));
      frame.run_app();
    }
  }

  if configuration.get_playername().eq("UnknownPlayer") {
    let mut frame = sciter::Window::new();
    let patcher : Arc<Mutex<Downloader>> = Arc::new(Mutex::new(Downloader::new()));
    frame.event_handler(Handler{patcher: patcher.clone(), configuration: configuration.clone()});
    frame.load_file(&format!("file://{}/dom/first-startup.htm", &current_dir));
    frame.run_app();
  }

  let game_location = configuration.get_game_location();
  let version_url = configuration.get_version_url();
  let launcher_theme = configuration.get_launcher_theme();
  
  info!("Launching sciter!");

  let mut frame = sciter::Window::new();
  let mut downloader = Downloader::new();
  downloader.set_location(game_location);
  downloader.set_version_url(version_url);
  let patcher : Arc<Mutex<Downloader>> = Arc::new(Mutex::new(downloader));
  frame.event_handler(Handler{patcher: patcher.clone(), configuration});
  frame.load_file(&format!("file://{}/{}/frontpage.htm", current_dir, &launcher_theme));
  info!("Launching app!");

  frame.run_app();

  info!("Gracefully shutting down app!");
  log::logger().flush();
  Ok(())
}
