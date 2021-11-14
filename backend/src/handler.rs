
use socket2::*;

use log::*;
use tokio::sync::Mutex;

use std::io::Write;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use renegadex_patcher::Patcher;
use sciter::Value;
use sha2::{Sha256, Digest};
use crate::configuration;
use crate::error::Error;
use crate::progress::ValueProgress;
use crate::version_information::VersionInformation;
use std::io::Read;
use ini::Ini;

/// The current launcher's version
static VERSION: &str = env!("CARGO_PKG_VERSION");

/// Structure for Sciter event handling.
pub(crate) struct Handler {
  /// The reference to the back-end library which is responsible for downloading and updating the game.
  pub patcher: Option<Patcher>,
  pub version_information: Arc<Mutex<Option<VersionInformation>>>,
  /// The configuration file for the launcher.
  pub configuration: configuration::Configuration,
  pub runtime: tokio::runtime::Handle
}

impl Handler {
  /// Check if there are game updates available, makes use of caching.
  fn check_update(&self, done: sciter::Value, error: sciter::Value) -> Result<(), Error> {
    let renegadex_location = self.configuration.get_game_location();
    let patch_dir_path = format!("{}/patcher/", renegadex_location).replace("//", "/");
    match std::fs::read_dir(patch_dir_path) {
      Ok(iter) => {
        if iter.count() != 0 {
          crate::spawn_wrapper::spawn(move || -> Result<(), Error> {done.call(None, &make_args!("resume"), None)?; Ok(()) });
          return Ok(());        }
      },
      Err(_e) => {}
    };
  
    let path = format!("{}/UDKGame/Config/DefaultRenegadeX.ini", renegadex_location);
    let conf = match Ini::load_from_file(&path) {
      Ok(file) => file,
      Err(_e) => {
        crate::spawn_wrapper::spawn(move || -> Result<(), Error> {done.call(None, &make_args!("full"), None)?; Ok(()) });
        return Ok(());      }
    };
  
    let section = conf.section(Some("RenX_Game.Rx_Game".to_owned())).ok_or_else(|| Error::None(format!("No Configuration section named \"RenX_Game.Rx_Game\"")))?;
    let game_version_number = section.get("GameVersionNumber").ok_or_else(|| Error::None(format!("No key in section \"RenX_Game.Rx_Game\"  named \"GameVersionNumber\"")))?;
    let version_information = self.version_information.clone();
    let version_url = self.configuration.get_version_url();
    crate::spawn_wrapper::spawn_async(&self.runtime, async {
      let version_information = version_information.lock().await;
      if version_information.is_none() {
        // download version information
        *version_information = Some(VersionInformation::retrieve(&version_url).await?);
      }
      let software_version = version_information.clone().unwrap().software;

      if software_version.version_number != game_version_number.parse::<u64>()? {
        crate::spawn_wrapper::spawn(move || -> Result<(), Error> { done.call(None, &make_args!("update"), None)?;; Ok(()) });
        return Ok(());
      }
      
      crate::spawn_wrapper::spawn(move || -> Result<(), Error> { done.call(None, &make_args!("up_to_date"), None)?; Ok(()) });
      return Ok(());
    });
    return Ok(());
  }

  /// Starts the downloading of the update/game
  fn start_download(&self, callback: sciter::Value, callback_done: sciter::Value, error: sciter::Value) -> Result<(), Error> {
    let progress = self.patcher.clone().lock().or_else(|_| Err(Error::MutexPoisoned(format!(""))))?.get_progress();
		crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
      let mut not_finished = true;
      let mut last_download_size : u64 = 0;
      while not_finished {
        std::thread::sleep(std::time::Duration::from_millis(500));

        let progress_locked = progress.lock().or_else(|_| Err(Error::MutexPoisoned(format!(""))))?;

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
        let me : Value = json.parse().or_else(|e| Err(Error::None(format!("Failed to parse Json, error \"{}\": {}", e, json))))?;
        last_download_size = progress_locked.download_size.0;
        not_finished = !progress_locked.finished_patching;
        drop(progress_locked);
        let callback_clone = callback.clone();
        crate::spawn_wrapper::spawn(move || -> Result<(), Error> {callback_clone.call(None, &make_args!(me), None)?; Ok(()) });
      }
      Ok(())
		});
    let patcher = self.patcher.clone();
    crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
      let result : Result<(), renegadex_patcher::Error>;
      {
        let mut locked_patcher = patcher.lock().or_else(|e| Err(Error::MutexPoisoned(format!("A poisoned Mutex: {}", e))))?;
        locked_patcher.rank_mirrors()?;
        locked_patcher.poll_progress();
        result = locked_patcher.download();
      }
      match result {
        Ok(()) => {
          info!("Calling download done");
          crate::spawn_wrapper::spawn(move || -> Result<(), Error> {callback_done.call(None, &make_args!(false,false), None)?; Ok(()) });
        },
        Err(e) => {
          error!("{:#?}", &e);
          crate::spawn_wrapper::spawn(move || -> Result<(), Error> {error.call(None, &make_args!(e.to_string()), None)?; Ok(()) });
        }
      };
      Ok(())
    });
    Ok(())
  }

  /// Removes files inside of the subdirectories that are not part of the instructions.json
  fn remove_unversioned(&self, callback_done: sciter::Value, error: sciter::Value) {
    let patcher = self.patcher.remove_unversioned();
    crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
      let result : Result<(), renegadex_patcher::Error>;
      {
        let mut locked_patcher = patcher.lock().or_else(|e| Err(Error::MutexPoisoned(format!("A poisoned Mutex: {}", e))))?;
        locked_patcher.rank_mirrors()?;
        result = locked_patcher.remove_unversioned();
      }
      match result {
        Ok(()) => {
          info!("Calling remove unversioned done");
          crate::spawn_wrapper::spawn(move || -> Result<(), Error> {callback_done.call(None, &make_args!("validate"), None)?; Ok(()) });
        },
        Err(e) => {
          error!("Error in remove_unversioned(): {:#?}", &e);
          crate::spawn_wrapper::spawn(move || -> Result<(), Error> {error.call(None, &make_args!(e.to_string()), None)?; Ok(()) });
        }
      };
      Ok(())
    });
  }

  fn get_video_location(&self, map_name: sciter::Value) -> String {
    self.configuration.get_video_location(map_name.to_string())
  }

  /// Retrieve the playername
  fn get_playername(&self) -> String {
    self.configuration.get_playername()
  }

  /// Set the playername
  fn set_playername(&self, username: sciter::Value) {
    self.configuration.set_playername(&username.as_string().expect(""))
  }

  /// Get Server List as plain text
  fn get_servers(&self, callback: sciter::Value) {
    crate::spawn_wrapper::spawn_async(&self.runtime, async move {
      let uri = "https://serverlist.ren-x.com/servers.jsp?id=launcher".parse::<download_async::http::Uri>()?;
      let mut downloader = download_async::Downloader::new();
      downloader.use_uri(uri);
      let headers = downloader.headers().expect("Could not unwrap headers");
      headers.append("User-Agent".parse::<download_async::http::header::HeaderName>().unwrap(), format!("RenX-Launcher ({})", VERSION).parse::<download_async::http::header::HeaderValue>().unwrap());

      let mut buffer = vec![];

      downloader.download(download_async::Body::empty(), &mut buffer).await?;

      crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
        let text : Value = std::str::from_utf8(&buffer).expect("Expected an utf-8 string").parse().expect(concat!(file!(),":",line!()));
        callback.call(None, &make_args!(text), None)?;
        Ok(())
      });
      Ok::<(), Error>(())
    });
  }

  /// Get ping of server
  fn get_ping(&self, server: sciter::Value, callback: sciter::Value) {
    crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
      let mut socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)).expect(concat!(file!(),":",line!(),": New socket"));
      let server_string = server.as_string().ok_or_else(|| Error::None(format!("Couldn't cast server \"{:?}\" to string", &server)))?;
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
      socket.send(&code)?;
      let mut buf : [u8; 100] = [0; 100];
      socket.set_read_timeout(Some(std::time::Duration::from_millis(500)))?;
      let result = socket.read(&mut buf);
      let elapsed = start_time.elapsed().as_millis() as i32;
      if result.is_ok() && buf[36..36+48] == code[16..] {
        crate::spawn_wrapper::spawn(move || -> Result<(), Error> {callback.call(None, &make_args!(server, elapsed), None)?; Ok(()) });
      }
      Ok(())
    });
  }

  /// Get the installed game's version
  fn get_game_version(&self) -> String {
    self.configuration.get_game_version()
  }

  /// Launch the game, if server variable it's value is "", then the game will be launched to the menu.
  fn launch_game(&self, server: Value, done: Value, error: Value) {
    let game_location = self.configuration.get_game_location();
    let launch_info =  self.configuration.get_launch_info();

    crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
      let server = server.as_string().ok_or_else(|| Error::None(format!("{}", concat!(file!(),":",line!()))))?;
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
          let output = child.wait()?;
          if output.success() {
            crate::spawn_wrapper::spawn(move || -> Result<(), Error> {done.call(None, &make_args!(), None)?; Ok(()) });
          } else {
            let code = output.code().ok_or_else(|| Error::None(format!("Couldn't get the exit code of the Game")))?;
            error!("The game exited in a crash: {}", code);
            crate::spawn_wrapper::spawn(move || -> Result<(), Error> {error.call(None, &make_args!(format!("The game exited in a crash: {}", code)), None)?; Ok(()) });
          }
        },
        Err(e) => {
          error!("Failed to open game: {}", &e);
          crate::spawn_wrapper::spawn(move || -> Result<(), Error> {error.call(None, &make_args!(format!("Failed to open game: {}", &e)), None)?; Ok(()) });
        }
      };
      Ok(())
    });
  }

  /// Gets the setting from the launchers configuration file.
  fn get_setting(&self, setting: sciter::Value) -> String {
    self.configuration.get_global_setting(&setting.as_string().expect(""))
  }

  /// Sets the setting in the launchers configuration file.
  fn set_setting(&self, setting: sciter::Value, value: sciter::Value) {
    self.configuration.set_global_setting(&setting.as_string().expect(""), &value.as_string().expect(""))
  }

  /// Get the current launcher version
  fn get_launcher_version(&self) -> &str {
    VERSION
  }

  /// Checks if the launcher is up to date
  fn check_launcher_update(&self, callback: Value) -> Result<(), Error> {
    let launcher_info_option = self.patcher.lock().or_else(|e| Err(Error::MutexPoisoned(format!("A mutex got poisoned: {}", e))))?.get_launcher_info();
    if let Some(launcher_info) = launcher_info_option {
      if VERSION != launcher_info.version_name && !launcher_info.prompted {
        crate::spawn_wrapper::spawn(move || -> Result<(), Error> {callback.call(None, &make_args!(launcher_info.version_name), None)?; Ok(()) });
      } else {
        crate::spawn_wrapper::spawn(move || -> Result<(), Error> {callback.call(None, &make_args!(Value::null()), None)?; Ok(()) });
      }
    } else {
      let patcher = self.patcher.clone();
      crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
        let mut patcher = patcher.lock().or_else(|e| Err(Error::MutexPoisoned(format!("A mutex got poisoned: {}", e))))?;
        patcher.retrieve_mirrors()?;
        let launcher_info_option = patcher.get_launcher_info();
        drop(patcher);
        if let Some(launcher_info) = launcher_info_option {
          if VERSION != launcher_info.version_name && !launcher_info.prompted {
            crate::spawn_wrapper::spawn(move || -> Result<(), Error> {callback.call(None, &make_args!(launcher_info.version_name), None)?; Ok(()) });
          } else {
            crate::spawn_wrapper::spawn(move || -> Result<(), Error> {callback.call(None, &make_args!(Value::null()), None)?; Ok(()) });
          }
        }
        Ok(())
      });
    }
    Ok(())
  }

  fn install_redists(&self, done: Value, error_callback: Value) -> Result<(), Error> {
    let mut cache_dir = dirs::cache_dir().ok_or_else(|| Error::None(format!("")))?;
    let patcher = self.patcher.clone();
    // Spawn thread, to not block the main process.
    crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
      cache_dir.set_file_name("UE3Redist.exe");
      let file = std::fs::File::create(&cache_dir)?;
      let mut patcher = patcher.lock().or_else(|e| Err(Error::MutexPoisoned(format!("A mutex got poisoned: {}", e))))?;
      patcher.rank_mirrors()?;
      let result = patcher.download_file_from_mirrors("/redists/UE3Redist.exe", file);
      drop(patcher);
      if let Err(error) = result {
        let error_string = format!("Failed to download UE3Redist: {}", error);
        crate::spawn_wrapper::spawn(move || -> Result<(), Error> {error_callback.call(None, &make_args!(error_string), None)?; Ok(()) });
        return Err(Error::PatcherError(error));
      }

      //run installer of UE3Redist and quit this.
      match runas::Command::new(cache_dir.to_str().ok_or_else(|| Error::None(format!("Failed to transform cache_dir to str")))?).gui(true).spawn() {
        Ok(mut child) => {
          match child.wait() {
            Ok(output) => {
              if output.success() {
                crate::spawn_wrapper::spawn(move || -> Result<(), Error> {done.call(None, &make_args!(), None)?; Ok(()) });
              } else {
                let code = output.code().ok_or_else(|| Error::None(format!("")))?;
                crate::spawn_wrapper::spawn(move || -> Result<(), Error> {error_callback.call(None, &make_args!(format!("UE3Redist.exe exited in a crash: {}", code)), None)?; Ok(()) });
              }
            },
            Err(e) => {
              error!("Failed to wait for UE3Redist: {}", &e);
              crate::spawn_wrapper::spawn(move || -> Result<(), Error> {error_callback.call(None, &make_args!(format!("Failed to wait for UE3Redist: {}", &e)), None)?; Ok(()) });
            }
          }
        },
        Err(e) => {
          // todo: the user might have cancelled the UAC dialog on purpose, ask if they want to continue the installation?
          error!("Failed to open UE3 Redistributables: {}", &e);
          crate::spawn_wrapper::spawn(move || -> Result<(), Error> {error_callback.call(None, &make_args!(format!("Failed to open UE3 Redistributables: {}", &e)), None)?; Ok(()) });
        }
      };

      Ok::<(), Error>(())
    });
    Ok(())
  }

  /// Launcher updater
  fn update_launcher(&self, progress: Value) -> Result<(), Error> {
    let launcher_info = self.patcher.lock().or_else(|e| Err(Error::MutexPoisoned(format!("A mutex got poisoned: {}", e))))?.get_launcher_info().ok_or_else(|| Error::None(format!("Couldn't fetch launcher info")))?;
    if VERSION != launcher_info.version_name {
      let socket_addrs = launcher_info.patch_url.parse::<url::Url>()?.socket_addrs(|| None)?;
      let uri = launcher_info.patch_url.parse::<download_async::http::Uri>()?;
      let good_hash = launcher_info.patch_hash.clone();
      drop(launcher_info);
      crate::spawn_wrapper::spawn_async(&self.runtime, async move {
        // Set up a request
        let mut downloader = download_async::Downloader::new();
        downloader.use_uri(uri);
        downloader.allow_http();
        downloader.use_sockets(socket_addrs.into());
        let value_progress = ValueProgress::new(progress.clone());
        downloader.use_progress(value_progress);
        downloader.headers().unwrap().append("User-Agent".parse::<download_async::http::header::HeaderName>().unwrap(), "sonny-launcher/1.0".parse::<download_async::http::header::HeaderValue>().unwrap());
        let mut buffer = vec![];
        downloader.download(download_async::Body::empty(), &mut buffer).await?;

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
        let mut output_path = std::env::current_exe()?;
        output_path.pop();
        let target_dir = output_path.clone();
        output_path.pop();
        let working_dir = output_path.clone();
        output_path.push("launcher_update_extracted/");
        info!("Extracting launcher update to: {:?}", output_path);
        let mut self_update_executor = output_path.clone();

        //extract files
        let result = unzip::Unzipper::new(download_contents, output_path).unzip().or_else(|e| Err(Error::UnzipError(e)))?;
        info!("{:#?}", result);
        
        //run updater program and quit this.
        self_update_executor.push("SelfUpdateExecutor.exe");
        let args = vec![format!("--pid={}",std::process::id()), format!("--target={}", target_dir.to_str().ok_or_else(|| Error::None(format!("Couldn't stringify target_dir")))?)];
        std::process::Command::new(self_update_executor)
                                    .current_dir(working_dir)
                                    .args(&args)
                                    .stdout(std::process::Stdio::piped())
                                    .stderr(std::process::Stdio::inherit())
                                    .spawn()?;
        std::process::exit(0);
        Ok::<(),Error>(())
      });
    }
    Ok(())
  }

  /// Fetch the text-resource at url with the specified headers.
  fn fetch_resource(&self, url: Value, mut headers_value: Value, callback: Value, context: Value) -> Result<(), Error> {
    headers_value.isolate();
    let mut downloader = download_async::Downloader::new();
    let headers = downloader.headers().expect("Couldn't get the headers of the request");

    for (key,value) in headers_value.items() {
      headers.insert(key.as_string().ok_or_else(|| Error::None(format!("Key value was empty.")))?.parse::<download_async::http::header::HeaderName>().unwrap(), value.as_string().ok_or_else(|| Error::None(format!("header value was empty.")))?.parse::<download_async::http::header::HeaderValue>().unwrap());
    }
    headers.insert("User-Agent".parse::<download_async::http::header::HeaderName>().unwrap(), format!("RenX-Launcher ({})", VERSION).parse::<download_async::http::header::HeaderValue>().unwrap());
    let uri = url.as_string().ok_or_else(|| Error::None(format!("Couldn't parse url as string.")))?.parse::<download_async::http::Uri>().unwrap();
    downloader.use_uri(uri);
    downloader.allow_http();

    crate::spawn_wrapper::spawn_async(&self.runtime, async move {
      let mut buffer = vec![];
      downloader.download(download_async::Body::empty(), &mut buffer).await?;
      crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
        let text = ::std::str::from_utf8(&buffer)?;
        callback.call(Some(context), &make_args!(text), None)?;
        Ok::<(), Error>(())
      });
      Ok::<(), Error>(())
    });
    Ok(())
  }

  /// Fetch the image at url with specified headers
  fn fetch_image(&self, url: Value, mut headers_value: Value, callback: Value, context: Value) -> Result<(), Error> {
    headers_value.isolate();
    let mut downloader = download_async::Downloader::new();
    let headers = downloader.headers().expect("Couldn't get the headers of the request");
    for (key,value) in headers_value.items() {
      headers.insert(key.as_string().ok_or_else(|| Error::None(format!("Key value was empty.")))?.parse::<download_async::http::header::HeaderName>().unwrap(), value.as_string().ok_or_else(|| Error::None(format!("header value was empty.")))?.parse::<download_async::http::header::HeaderValue>().unwrap());
    }
    headers.insert("User-Agent".parse::<download_async::http::header::HeaderName>().unwrap(), format!("RenX-Launcher ({})", VERSION).parse::<download_async::http::header::HeaderValue>().unwrap());
    let uri = url.as_string().ok_or_else(|| Error::None(format!("Couldn't parse url as string.")))?.parse::<download_async::http::Uri>()?;
    downloader.use_uri(uri);
    downloader.allow_http();

    crate::spawn_wrapper::spawn_async(&self.runtime, async move {
      let mut buffer = vec![];

      downloader.download(download_async::Body::empty(), &mut buffer).await?;
      crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
        let image = sciter::graphics::Image::load(&buffer).ok();
        if let Some(image) = image {
          info!("Sending back image as it was succesfull");
          callback.call(Some(context), &make_args!(image), None)?;
        } else {
          info!("Failed to load bytes as an image");
          callback.call(Some(context), &make_args!(), None)?;
        }
        Ok(())
      });
      Ok::<(), Error>(())
    });
    Ok(())
  }

  fn open_launcher_logs_folder(&self) {
    let spawned_process = std::process::Command::new("explorer.exe").arg(self.configuration.get_log_directory()).spawn();
  }
}

impl sciter::EventHandler for Handler {
  fn get_subscription(&mut self) -> Option<sciter::dom::event::EVENT_GROUPS> {
    Some(sciter::dom::event::default_events() | sciter::dom::event::EVENT_GROUPS::HANDLE_METHOD_CALL)
  }

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
    fn open_launcher_logs_folder();

    fn check_launcher_update(Value);
    fn update_launcher(Value);
    fn fetch_resource(Value,Value,Value,Value);
    fn fetch_image(Value,Value,Value,Value);
    fn get_video_location(Value);
  }

  fn on_script_call(&mut self, root: sciter::HELEMENT, name: &str, argv: &[Value]) -> Option<Value> {
      let args = argv.iter().map(|x| format!("{:?}", &x)).collect::<Vec<String>>().join(", ");
      
      info!("Called {}({}) from element: {:?}", name, args, sciter::Element::from(root));
      let handled = self.dispatch_script_call(root, name, argv);
      if handled.is_some() {
        info!("End {}({}): {:?}", name, args, handled);
        return handled;
      }
      error!("{}({}) does not exist!", name, args);
      None
  }
}