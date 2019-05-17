#![windows_subsystem="windows"]

extern crate hyper;
extern crate tokio;
extern crate tokio_reactor;
#[macro_use] extern crate futures;
#[macro_use] extern crate sciter;
extern crate renegadex_patcher;
extern crate ini;
extern crate irc;
extern crate single_instance;
extern crate socket2;
extern crate rand;
extern crate deunicode;
extern crate percent_encoding;
extern crate unzip;

use std::sync::{Arc,Mutex};

use sciter::Value;

use socket2::*;

use renegadex_patcher::{Downloader,Update, traits::Error};
use ini::Ini;
use irc::client::prelude::*;
use single_instance::SingleInstance;
use hyper::rt::Future;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::io::Write;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

struct Handler {
  patcher: Arc<Mutex<Downloader>>,
  irc_client: Arc<Mutex<Option<irc::client::IrcClient>>>,
  irc_callback: Arc<Mutex<Option<sciter::Value>>>,
  conf: Arc<Mutex<ini::Ini>>,
}

impl Handler {
  fn check_update(&self, done: sciter::Value, error: sciter::Value) {
    {
      let progress = self.patcher.clone().lock().unwrap().get_progress();
      let update = &progress.lock().unwrap().update;
      match update {
        Update::UpToDate => {
          std::thread::spawn(move || {done.call(None, &make_args!(false, false), None).unwrap();});
          return;
        },
        Update::Resume | Update::Full => {
          std::thread::spawn(move || {done.call(None, &make_args!(true, true), None).unwrap();});
          return;
        },
        Update::Delta => {
          std::thread::spawn(move || {done.call(None, &make_args!(true, false), None).unwrap();});
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
          let mut patcher = patcher.lock().unwrap();
          patcher.retrieve_mirrors()?;
          update_available = patcher.update_available()?;
        }
        match update_available {
          Update::UpToDate => {
            std::thread::spawn(move || {done.call(None, &make_args!(false, false), None).unwrap();});
          },
          Update::Resume | Update::Full => {
            std::thread::spawn(move || {done.call(None, &make_args!(true, true), None).unwrap();});
          },
          Update::Delta => {
            std::thread::spawn(move || {done.call(None, &make_args!(true, false), None).unwrap();});
          },
          Update::Unknown => {
            eprintln!("Update::Unknown");
          }
        };
        Ok(())
		  };
      let result : Result<(),Error> = check_update();
      if result.is_err() {
        use std::error::Error;
        let err = result.unwrap_err();
        eprintln!("{:#?}", err.description());
        std::thread::spawn(move || {error.call(None, &make_args!(err.description()), None).unwrap();});
      }
    });
  }

  fn start_download(&self, callback: sciter::Value, callback_done: sciter::Value, error: sciter::Value) {
    let progress = self.patcher.clone().lock().unwrap().get_progress();
		std::thread::spawn(move || {
      let mut not_finished = true;
      let mut last_download_size : u64 = 0;
      while not_finished {
        std::thread::sleep(std::time::Duration::from_millis(500));
        {
          let progress_locked = progress.lock().unwrap();
          let me : Value = format!(
            "{{\"hash\": [{},{}],\"download\": [{},{}],\"patch\": [{},{}],\"download_speed\": {}}}",
            progress_locked.hashes_checked.0.clone(),
            progress_locked.hashes_checked.1.clone(),
            progress_locked.download_size.0.clone()/10000,
            progress_locked.download_size.1.clone()/10000,
            progress_locked.patch_files.0.clone(),
            progress_locked.patch_files.1.clone(),
            (progress_locked.download_size.0 - last_download_size) as f64 / 500000.0
          ).parse().unwrap();
          last_download_size = progress_locked.download_size.0.clone();
          not_finished = !progress_locked.finished_patching;
          let callback_clone = callback.clone();
          std::thread::spawn(move || {callback_clone.call(None, &make_args!(me), None).unwrap();});
        }
      }
		});
    let patcher = self.patcher.clone();
    std::thread::spawn(move || {
      let result : Result<(), renegadex_patcher::traits::Error>;
      {
        let mut locked_patcher = patcher.lock().unwrap();
        locked_patcher.poll_progress();
        result = locked_patcher.download();
      }
      match result {
        Ok(()) => {
          println!("Calling download done");
          std::thread::spawn(move || {callback_done.call(None, &make_args!(false,false), None).unwrap();});
        },
        Err(e) => {
          use std::error::Error;
          eprintln!("{:#?}", e.description());
          std::thread::spawn(move || {error.call(None, &make_args!(e.description()), None).unwrap();});
        }
      };
    });
  }

  fn remove_unversioned(&self, callback_done: sciter::Value, error: sciter::Value) {
    let patcher = self.patcher.clone();
    std::thread::spawn(move || {
      let result : Result<(), renegadex_patcher::traits::Error>;
      {
        let mut locked_patcher = patcher.lock().unwrap();
        result = locked_patcher.remove_unversioned();
      }
      match result {
        Ok(()) => {
          println!("Calling remove unversioned done");
          std::thread::spawn(move || {callback_done.call(None, &make_args!(false,false), None).unwrap();});
        },
        Err(e) => {
          use std::error::Error;
          eprintln!("Error in remove_unversioned(): {:#?}", e.description());
          std::thread::spawn(move || {error.call(None, &make_args!(e.description()), None).unwrap();});
        }
      };
    });
  }

  fn send_irc_message(&self, message: sciter::Value) {
    match *self.irc_client.lock().unwrap() {
      Some(ref irc_client) => irc_client.send_privmsg("#renegadex", message.as_string().unwrap()).unwrap(),
      None => {}
    }
  }

  fn register_irc_callback(&self, callback: sciter::Value) {
    let mut irc_callback = self.irc_callback.lock().unwrap();
    *irc_callback = Some(callback.clone());
  }

  fn get_playername(&self) -> String {
    let conf_unlocked = self.conf.clone();
    let conf = conf_unlocked.lock().unwrap();
    let section = conf.section(Some("RenX_Launcher".to_owned())).unwrap();
    section.get("PlayerName").unwrap().to_string()
  }

  fn set_playername(&self, username: sciter::Value) {
    let conf_unlocked = self.conf.clone();
    let mut conf = conf_unlocked.lock().unwrap();
    let mut section = conf.with_section(Some("RenX_Launcher".to_owned()));
    section.set("PlayerName", username.as_string().unwrap());
    conf.write_to_file("RenegadeX-Launcher.ini").unwrap();
  }

  fn get_irc_nick(&self) -> String {
    let conf_unlocked = self.conf.clone();
    let conf = conf_unlocked.lock().unwrap();
    let section = conf.section(Some("RenX_Launcher".to_owned())).unwrap();
    section.get("IrcNick").unwrap().to_string()
  }

  fn set_irc_nick(&self, nick: sciter::Value) {
    let conf_unlocked = self.conf.clone();
    let mut conf = conf_unlocked.lock().unwrap();
    let mut section = conf.with_section(Some("RenX_Launcher".to_owned()));
    section.set("IrcNick", nick.as_string().unwrap());
    conf.write_to_file("RenegadeX-Launcher.ini").unwrap();
    match *self.irc_client.lock().unwrap() {
      Some(ref irc_client) => irc_client.send(Command::NICK(nick.as_string().unwrap())).unwrap(),
      None => {}
    }
  }

  fn get_servers(&self, callback: sciter::Value) {
    std::thread::spawn(move || {
      let mut future;
      {
        let url = "http://serverlist.renegade-x.com/servers.jsp".parse::<hyper::Uri>().unwrap();
        let host_port = format!("{}:{}",url.host().unwrap(),url.port_u16().unwrap_or(80_u16));
        let tcpstream = std::net::TcpStream::connect(host_port).unwrap();
        future = tokio::net::TcpStream::from_std(tcpstream, &tokio_reactor::Handle::default()).map(|tcp| {
          hyper::client::conn::handshake(tcp)
        }).unwrap().and_then(move |(mut client, conn)| {
          let mut req = hyper::Request::builder();
          req.uri(url.path()).header("host", url.host().unwrap()).header("User-Agent", "sonny-launcher/1.0");
          let req = req.body(hyper::Body::empty()).unwrap();
          let res = client.send_request(req).and_then(|res| {
            use hyper::rt::*;
            let abort_in_error = res.status() != 200 && res.status() != 206;
            res.into_body().concat2()
          }).and_then(move |body| {
            std::thread::spawn(move || {
              let text : Value = ::std::str::from_utf8(&body).expect("Expected an utf-8 string").parse().unwrap();
              callback.call(None, &make_args!(text), None).unwrap();
            });
            Ok(())
          });
          // Put in an Option so poll_fn can return it later
          let mut conn = Some(conn);
          let until_upgrade = futures::future::poll_fn(move || {
            try_ready!(conn.as_mut().unwrap().poll_without_shutdown());
            Ok(futures::Async::Ready(conn.take().unwrap()))
          });

          res.join(until_upgrade)
        });
      }
      tokio::runtime::current_thread::Runtime::new().unwrap().block_on(future).unwrap();
    });
  }

  fn get_ping(&self, server: sciter::Value, callback: sciter::Value) {
    std::thread::spawn(move || {
      let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4())).unwrap();
      use std::str::FromStr;
      let sock_addr = std::net::SocketAddr::from_str(&server.as_string().unwrap()).unwrap().into();
      let start_time = std::time::Instant::now();
      socket.connect_timeout(&sock_addr, std::time::Duration::from_millis(500)).unwrap();
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
      socket.send(&code).unwrap();
      let mut buf : [u8; 100] = [0; 100];
      socket.set_read_timeout(Some(std::time::Duration::from_millis(500))).unwrap();
      socket.recv(&mut buf).unwrap();
      let elapsed = start_time.elapsed().as_millis() as i32;
      if buf[36..36+48] == code[16..] {
        //println!("{:#?}", &elapsed);
        std::thread::spawn(move || {callback.call(None, &make_args!(server, elapsed), None).unwrap();});
      } else {
        //println!("{:?}", &buf[36..36+48]);
        //println!("{:?}", &code[16..]);
      }
    });
  }


  fn get_game_version(&self) -> String {
    let conf = self.conf.lock().unwrap();
    let section = conf.section(Some("RenX_Launcher".to_owned())).unwrap();
    let game_location = section.get("GameLocation").unwrap().clone();
    match Ini::load_from_file(format!("{}/UDKGame/Config/DefaultRenegadeX.ini", game_location)) {
      Ok(conf) => {
        let section = conf.section(Some("RenX_Game.Rx_Game".to_owned())).unwrap();
        section.get("GameVersion").unwrap().clone()
      },
      Err(_e) => {
        "Not installed".to_string()
      }
    }
  }

  fn launch_game(&self, server: Value, done: Value, error: Value) {
    let conf = self.conf.lock().unwrap();
    let section = conf.section(Some("RenX_Launcher".to_owned())).unwrap();
    let game_location = section.get("GameLocation").unwrap().clone();
    let playername = section.get("PlayerName").unwrap().clone();
    let startup_movie_disabled = section.get("skipMovies").unwrap().clone() == "true";
    let bit_version = if section.get("64-bit-version").unwrap().clone() == "true" { "64" } else { "32" };
    drop(conf);
    std::thread::spawn(move || {
      let mut args = vec![server.as_string().unwrap(), format!("-ini:UDKGame:DefaultPlayer.Name={}", playername)];
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
            std::thread::spawn(move || {done.call(None, &make_args!(), None).unwrap();});
          } else {
            //eprintln!("{:#?}", output.unwrap_err().description());
            std::thread::spawn(move || {error.call(None, &make_args!("The game exited in a crash"), None).unwrap();});
          }
        },
        Err(e) => {
          use std::error::Error;
          eprintln!("Failed to create child: {}", e.description());
          std::thread::spawn(move || {error.call(None, &make_args!(format!("Failed to open game: {}", e.description())), None).unwrap();});
        }
      };
    });
  }

  fn get_setting(&self, setting: sciter::Value) -> String {
    let conf_unlocked = self.conf.clone();
    let conf = conf_unlocked.lock().unwrap();
    let section = conf.section(Some("RenX_Launcher".to_owned())).unwrap();
    section.get(&setting.as_string().unwrap()).unwrap().to_string()
  }
  fn set_setting(&self, setting: sciter::Value, value: sciter::Value) {
    let conf_unlocked = self.conf.clone();
    let mut conf = conf_unlocked.lock().unwrap();
    let mut section = conf.with_section(Some("RenX_Launcher".to_owned()));
    section.set(setting.as_string().unwrap(), value.as_string().unwrap());
    conf.write_to_file("RenegadeX-Launcher.ini").unwrap();
  }

  fn deunicode(&self, string: Value) -> String {
    deunicode::deunicode(&string.as_string().unwrap())
  }

  fn get_launcher_version(&self) -> &str {
    return VERSION;
  }

  fn check_launcher_update(&self, callback: Value) {
    let launcher_info = self.patcher.lock().unwrap().get_launcher_info().unwrap();
    if VERSION != launcher_info.version_name && !launcher_info.prompted {
      std::thread::spawn(move || {callback.call(None, &make_args!(launcher_info.version_name), None).unwrap();});
    }
  }

  fn update_launcher(&self, progress: Value) {
    let launcher_info = self.patcher.lock().unwrap().get_launcher_info().unwrap();
    if VERSION != launcher_info.version_name {
      std::thread::spawn(move || {
        let mut future;
        let mut download_contents = Arc::new(Mutex::new(Vec::new()));
        let download_contents_clone = download_contents.clone();
        {
          //let bw = std::io::BufWriter::new();
          let url = launcher_info.patch_url.parse::<hyper::Uri>().unwrap();
          let host_port = format!("{}:{}",url.host().unwrap(),url.port_u16().unwrap_or(80_u16));
          let tcpstream = std::net::TcpStream::connect(host_port).unwrap();
          future = tokio::net::TcpStream::from_std(tcpstream, &tokio_reactor::Handle::default()).map(|tcp| {
            hyper::client::conn::handshake(tcp)
          }).unwrap().and_then(move |(mut client, conn)| {
            let mut req = hyper::Request::builder();
            req.uri(url.path()).header("host", url.host().unwrap()).header("User-Agent", "sonny-launcher/1.0");
            let req = req.body(hyper::Body::empty()).unwrap();
            let res = client.send_request(req).and_then(move |res| {
              use hyper::rt::*;
              let abort_in_error = res.status() != 200 && res.status() != 206;
              let content_length : usize = res.headers().get("content-length").unwrap().to_str().unwrap().parse().unwrap();
              let progress_clone = progress.clone();
              std::thread::spawn(move || {progress.call(None, &make_args!(Value::null(), content_length as i32), None).unwrap();});
              //println!("{:?}", res.headers().get("content-length").unwrap());
              *download_contents_clone.lock().unwrap() = Vec::with_capacity(content_length);

              res.into_body().for_each(move |chunk| {
                let chunk_size : i32 = chunk.len() as i32;
                let progress_clone = progress_clone.clone();
                std::thread::spawn(move || {progress_clone.call(None, &make_args!(chunk_size, Value::null()), None).unwrap();});
                download_contents_clone.lock().unwrap().write_all(&chunk).map_err(|e| panic!("Writer encountered an error: {}", e))
              })
            });
            // Put in an Option so poll_fn can return it later
            let mut conn = Some(conn);
            let until_upgrade = futures::future::poll_fn(move || {
              try_ready!(conn.as_mut().unwrap().poll_without_shutdown());
              Ok(futures::Async::Ready(conn.take().unwrap()))
            });
            res.join(until_upgrade)
          });
        }
        tokio::runtime::current_thread::Runtime::new().unwrap().block_on(future).unwrap();
        let download_contents = std::io::Cursor::new(Arc::try_unwrap(download_contents).unwrap().into_inner().unwrap());
        let mut output_path = std::env::current_exe().unwrap();
        output_path.pop();
        let target_dir = output_path.clone();
        output_path.pop();
        output_path.push("launcher_update_extracted/");
        println!("{:?}", output_path);
        let mut SUE = output_path.clone();
        unzip::Unzipper::new(download_contents, output_path).unzip().unwrap();
        let working_dir = SUE.clone();
        SUE.push("SelfUpdateExecutor.exe");
        let mut args = vec![format!("--pid={}",std::process::id()), format!("--target={}", target_dir.to_str().unwrap())];
        std::process::Command::new(SUE)
                                     .current_dir(working_dir)
                                     .args(&args)
                                     .stdout(std::process::Stdio::piped())
                                     .stderr(std::process::Stdio::inherit())
                                     .spawn().unwrap();
        std::process::exit(0);
        //download file
        //extract files
        //run updater program and quit this.
      });
    }
  }
}

impl sciter::EventHandler for Handler {
	dispatch_script_call! {
		fn check_update(Value, Value);
    fn start_download(Value, Value, Value);
    fn remove_unversioned(Value, Value);
    fn send_irc_message(Value); //Parameter is a string
    fn register_irc_callback(Value); //Register's the callback
     //removed funtion of what I've forgot what it was intended for, atleast three values should be differentiated: UpToDate, Downloading, UpdateAvailable
    fn get_playername();
    fn get_irc_nick();
    fn get_game_version();
    fn set_playername(Value);
    fn set_irc_nick(Value);
    fn get_servers(Value);
    fn launch_game(Value, Value, Value); //Parameters: (Server IP+Port, onDone, onError);
    fn get_ping(Value, Value);
    fn deunicode(Value);
    fn get_setting(Value);
    fn set_setting(Value, Value);
    fn get_launcher_version();
    fn check_launcher_update(Value);
    fn update_launcher(Value);
  }
}

fn main() {
  let instance = SingleInstance::new("RenegadeX-Launcher").unwrap();
  assert!(instance.is_single());

  let conf = match Ini::load_from_file("RenegadeX-Launcher.ini") {
    Ok(conf) => conf,
    Err(_e) => {
      //TODO spawn dialog that gets PlayerName
      let mut conf = Ini::new();
      conf.with_section(Some("RenX_Launcher"))
        .set("GameLocation", "C:/Program Files (x86)/Renegade X/")
        .set("VersionUrl", "https://static.renegade-x.com/launcher_data/version/release.json")
        .set("PlayerName", "UnknownPlayer")
        .set("IrcNick", "UnknownPlayer")
        .set("LauncherTheme", "dom")
        .set("LastNewsGUID", "")
        .set("64-bit-version", "true")
        .set("skipMovies", "false");
      let conf_arc = Arc::new(Mutex::new(conf.clone()));
      {
        sciter::set_options(
          sciter::RuntimeOptions::ScriptFeatures(
            sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SOCKET_IO as u8 | // Enables connecting to the inspector via Ctrl+Shift+I
            sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_EVAL as u8  // Enables execution of Eval inside of TI-Script
          )
        ).unwrap(); 
        let mut frame = sciter::Window::new();
        let patcher : Arc<Mutex<Downloader>> = Arc::new(Mutex::new(Downloader::new()));
        let client_clone : Arc<Mutex<Option<irc::client::IrcClient>>> = Arc::new(Mutex::new(None));
        let irc_messages : Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let irc_callback : Arc<Mutex<Option<sciter::Value>>> = Arc::new(Mutex::new(None));
        frame.event_handler(Handler{patcher: patcher.clone(), irc_client: client_clone.clone(), irc_callback: irc_callback.clone(), conf: conf_arc.clone()});
        let mut current_path = std::env::current_exe().unwrap();
        current_path.pop();
        current_path.push(format!("dom/first-startup.htm"));
        frame.load_file(&percent_encoding::utf8_percent_encode(current_path.to_str().unwrap(), percent_encoding::DEFAULT_ENCODE_SET).to_string());
        frame.run_app();
      }
      //conf.write_to_file("RenegadeX-Launcher.ini").unwrap();
      conf = match Arc::try_unwrap(conf_arc) {
        Ok(conf_mutex) => {
          conf_mutex.into_inner().unwrap().clone()
        },
        Err(_e) => {
          panic!("No way to deal with this for now");
        }
      };
      conf
    }
  };

  let section = conf.section(Some("RenX_Launcher".to_owned())).unwrap();
  let game_location = section.get("GameLocation").unwrap();
  let version_url = section.get("VersionUrl").unwrap();
  let launcher_theme = section.get("LauncherTheme").unwrap();
  let irc_name = section.get("IrcNick").unwrap().clone();

  let mut current_path = std::env::current_exe().unwrap();
  current_path.pop();
  sciter::set_options(
    sciter::RuntimeOptions::ScriptFeatures(
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SOCKET_IO as u8 | // Enables connecting to the inspector via Ctrl+Shift+I
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_EVAL as u8  // Enables execution of Eval inside of TI-Script
    )
  ).unwrap(); 
  let mut frame = sciter::Window::new();
  let mut downloader = Downloader::new();
  downloader.set_location(game_location.to_string());
  downloader.set_version_url(version_url.to_string());
  let patcher : Arc<Mutex<Downloader>> = Arc::new(Mutex::new(downloader));
  let client_clone : Arc<Mutex<Option<irc::client::IrcClient>>> = Arc::new(Mutex::new(None));
  let irc_messages : Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
  let irc_callback : Arc<Mutex<Option<sciter::Value>>> = Arc::new(Mutex::new(None));
  let conf_arc = Arc::new(Mutex::new(conf.clone()));
  frame.event_handler(Handler{patcher: patcher.clone(), irc_client: client_clone.clone(), irc_callback: irc_callback.clone(), conf: conf_arc});
  current_path.push(format!("{}/frontpage.htm", launcher_theme));
  println!("{:?}", &current_path);
  frame.load_file(&percent_encoding::utf8_percent_encode(current_path.to_str().unwrap(), percent_encoding::DEFAULT_ENCODE_SET).to_string());

  let irc_thread = std::thread::spawn(move || {
    let config = Config {
      nickname: Some(irc_name.to_owned()),
      alt_nicks: Some(vec![format!("{}_", &irc_name)]),
      server: Some("irc.cncirc.net".to_owned()),
      channels: Some(vec!["#renegadex".to_owned()]),
      use_ssl: Some(true),
      ..Config::default()
    };
    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();
    {
      let mut client_lock = client_clone.lock().unwrap();
      *client_lock = Some(client.clone());
    }
    reactor.register_client_with_handler(client, move |client, event| {
      if let Command::PRIVMSG(channel, message) = &event.command {
        if channel == "#renegadex" {
          let ui_option = irc_callback.lock().unwrap();
          match *ui_option {
            Some(ref ui) => {
              let username = event.prefix.unwrap();
              let username = username.split("!").nth(0).unwrap();
              ui.call(None, &make_args!(username,message.as_str()), None).unwrap();
            },
            None => {
              println!("{:#?}", &message);
            }
          }
        }
      }
      // And here we can do whatever we want with the messages.
      Ok(())
    });
    reactor.run().unwrap();
  });
  frame.run_app();
}

/*
pub struct Launcher {
  //for example: ~/RenegadeX/
  RenegadeX_location: Option<String>,
  //For example: DRI_PRIME=1
  env_arguments: Option<String>,
  player_name: Option<String>,
  servers: Option<json::JsonValue>,
  ping: Option<json::JsonValue>,
  x64_bit: bool
}

impl Launcher {
  pub fn new(game_folder: String) -> Launcher {
    Launcher {
      RenegadeX_location: Some(game_folder),
      env_arguments: None,
      player_name: None,
      servers: None,
      ping: None,
      x64_bit: true
    }
  }

  pub fn refresh_server_list(&mut self) {
    
  }

  pub fn launch_game(&mut self, server_index: Option<u16>) -> std::process::Child {
    if server_index == None {
      let mut wine_location = self.RenegadeX_location.clone().unwrap();
      wine_location.push_str("libs/wine/bin/wine64");
      let mut game_location = self.RenegadeX_location.clone().unwrap();
      game_location.push_str("game_files/Binaries/Win64/UDK.exe");

      let mut wine_prefix = self.RenegadeX_location.clone().unwrap();
      wine_prefix.push_str("wine_instance/");
      return process::Command::new(wine_location)
        .arg(game_location)
        //.arg("5.39.74.177:7777")
        .arg("-nomoviestartup")
        .arg("-ini:UDKGame:DefaultPlayer.Name=SonnyX")	
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::inherit())
        .spawn().expect("failed to execute child");

    } else {
      let mut game_location = self.RenegadeX_location.clone().unwrap();
      game_location.push_str("C:/Program Files (x86)/Renegade X/Binaries/Win64/UDK.exe");
      return process::Command::new(game_location)
        .arg("some server")
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::inherit())
        .spawn().expect("failed to execute child");
    }
  }
}
*/
