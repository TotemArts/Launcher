#![windows_subsystem="console"]

extern crate reqwest;
extern crate json;
#[macro_use]
extern crate sciter;
extern crate renegadex_patcher;
extern crate xml;
extern crate ini;
extern crate irc;
extern crate single_instance;
extern crate chrono;
#[cfg(unix)]
extern crate gag;

mod traits;
#[cfg(windows)]
pub mod redirect;

use std::sync::{Arc,Mutex};

use sciter::Value;

use renegadex_patcher::{Downloader,Update, traits::Error};
use ini::Ini;
use irc::client::prelude::*;
use single_instance::SingleInstance;

#[cfg(windows)]
fn redirect_std(output_filename: String) {
  std::thread::spawn(move || {
    let mut stdout = redirect::stdout().unwrap();
    //let mut stderr = redirect::stderr().unwrap();
    let mut stderr = std::io::stderr();
    use std::io::{Write,Read,Seek};
    loop {
      let mut output_file = std::fs::OpenOptions::new().read(true).write(true).create(true).open(output_filename.clone()).unwrap();
      output_file.seek(std::io::SeekFrom::End(0)).unwrap();
      let mut buf = Vec::new();
      stdout.read_to_end(&mut buf).unwrap();
      //stderr.read_to_end(&mut buf).unwrap();
      output_file.write_all(&buf).unwrap();
      stderr.write_all(&buf).unwrap();
    }
  });
}

#[cfg(unix)]
fn redirect_std(output_filename: String) {
  let file = std::fs::OpenOptions::new().read(true).write(true).create(true).open(output_filename).unwrap();
  std::thread::spawn(move || {
    let mut stdout = gag::Redirect::stdout(file.try_clone().unwrap()).unwrap();
    let mut stderr = gag::Redirect::stderr(file).unwrap();
    loop {
      std::thread::sleep(std::time::Duration::from_millis(500));
    }
  });
}

struct Handler {
  patcher: Arc<Mutex<Downloader>>,
  irc_client: Arc<Mutex<Option<irc::client::IrcClient>>>,
  irc_callback: Arc<Mutex<Option<sciter::Value>>>,
  conf: Arc<Mutex<ini::Ini>>,
}

impl Handler {
  fn check_update(&self, done: sciter::Value, error: sciter::Value) -> bool {
    {
      let progress = self.patcher.clone().lock().unwrap().get_progress();
      let update = &progress.lock().unwrap().update;
      match update {
        Update::UpToDate => {
          println!("No update available");
          done.call(None, &make_args!(false, false), None).unwrap();
          return true;
        },
        Update::Resume | Update::Full => {
          println!("Resuming download!");
          done.call(None, &make_args!(true, true), None).unwrap();
          return true;
        },
        Update::Delta => {
          println!("Update available");
          done.call(None, &make_args!(true, false), None).unwrap();
          return true;
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
            println!("No update available");
            done.call(None, &make_args!(false, false), None).unwrap();
          },
          Update::Resume | Update::Full => {
            println!("Resuming download!");
            done.call(None, &make_args!(true, true), None).unwrap();
          },
          Update::Delta => {
            println!("Update available");
            done.call(None, &make_args!(true, false), None).unwrap();
          },
          Update::Unknown => {
            println!("Impossible");
          }
        };
        Ok(())
		  };
      let result : Result<(),Error> = check_update();
      if result.is_err() {
        //println!("{:#?}", result.unwrap_err().description());
        use std::error::Error;
        error.call(None, &make_args!(result.unwrap_err().description()), None).unwrap();
      }
    });
		true
  }

  fn start_download(&self, callback: sciter::Value, callback_done: sciter::Value, error: sciter::Value) -> bool {
    let progress = self.patcher.clone().lock().unwrap().get_progress();
		std::thread::spawn(move || {
      let mut not_finished = true;
      while not_finished {
        std::thread::sleep(std::time::Duration::from_millis(500));
        {
          let progress_locked = progress.lock().unwrap();
          let me : Value = format!(
            "{{\"hash\": [{},{}],\"download\": [{},{}],\"patch\": [{},{}]}}",
            progress_locked.hashes_checked.0.clone(),
            progress_locked.hashes_checked.1.clone(),
            progress_locked.download_size.0.clone()/10000,
            progress_locked.download_size.1.clone()/10000,
            progress_locked.patch_files.0.clone(),
            progress_locked.patch_files.1.clone()
          ).parse().unwrap();
          not_finished = !progress_locked.finished_patching;
          callback.call(None, &make_args!(me), None).unwrap();
        }
      }
		});
    let patcher = self.patcher.clone();
    std::thread::spawn(move || {
      match patcher.lock().unwrap().download() {
        Ok(()) => {
          callback_done.call(None, &make_args!(false,false), None).unwrap();
        },
        Err(e) => {
          use std::error::Error;
          error.call(None, &make_args!(e.description()), None).unwrap();
        }
      };
    });
    true
  }

  fn send_irc_message(&self, message: sciter::Value) -> bool {
    match *self.irc_client.lock().unwrap() {
      Some(ref irc_client) => irc_client.send_privmsg("#renegadex", message.as_string().unwrap()).unwrap(),
      None => {}
    }
    true
  }

  fn register_irc_callback(&self, callback: sciter::Value) -> bool {
    println!("registering irc_callback: {:#?}", &callback);
    let mut irc_callback = self.irc_callback.lock().unwrap();
    *irc_callback = Some(callback.clone());
    true
  }
  fn get_status(&self, callback: sciter::Value) -> bool {
    let progress = self.patcher.clone().lock().unwrap().get_progress();
    std::thread::spawn(move || {
      let progress_locked = progress.lock().unwrap();
      callback.call(None, &make_args!(progress_locked.finished_patching), None).unwrap();
    });
    true
  }

  fn get_playername(&self, callback: sciter::Value) -> bool {
    let conf_unlocked = self.conf.clone();

    std::thread::spawn(move || {
      let conf = conf_unlocked.lock().unwrap();
      let section = conf.section(Some("RenX_Launcher".to_owned())).unwrap();
      let playername = section.get("PlayerName").unwrap();
      callback.call(None, &make_args!(playername.as_str()), None).unwrap();
    });
    true
  }

  fn get_mirrors(&self, callback: sciter::Value) -> bool {
    std::thread::spawn(move || {
      //reqwest server
      let text : Value = reqwest::get("http://serverlist.renegade-x.com/servers.jsp").unwrap().text().unwrap().parse().unwrap();
      callback.call(None, &make_args!(text), None).unwrap();
    });
    true
  }

  fn launch_game(&self, server: Value, done: Value, error: Value) {
    let conf = self.conf.lock().unwrap();
    let section = conf.section(Some("RenX_Launcher".to_owned())).unwrap();
    let game_location = section.get("GameLocation").unwrap().clone();
    let bit_version = section.get("64-bit-version").unwrap().clone();
    drop(conf);
    std::thread::spawn(move || {
      let mut game_instance = std::process::Command::new(format!("{}/Binaries/Win{}/UDK.exe", game_location, if bit_version == "true" { "64" } else { "32" }))
        .arg(server.as_string().unwrap())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn().expect("failed to execute child");
      let output = game_instance.wait().expect("Failed to wait on game-instance to finish");
      if output.success() {
        done.call(None, &make_args!(), None);
      } else {
        done.call(None, &make_args!(), None);
      }
      
    });
  }
}

impl sciter::EventHandler for Handler {
	dispatch_script_call! {
		fn check_update(Value, Value);
    fn start_download(Value, Value, Value);
    fn send_irc_message(Value); //Parameter is a string
    fn register_irc_callback(Value); //Register's the callback
    fn get_status(Value); //forgot what it was intended for, atleast two three values should be differentiated: UpToDate, Downloading, UpdateAvailable
    fn get_playername(Value);
    fn get_mirrors(Value);
    fn launch_game(Value, Value, Value); //Parameters: (Server IP+Port, onDone, onError);
  }
}

fn main() {
  let instance = SingleInstance::new("RenegadeX-Launcher").unwrap();
  assert!(instance.is_single());

  //let output_filename = format!("{}.output", chrono::Utc::now().format("%v_%X")).replace(":", "-");
  //redirect_std(output_filename);

  let conf = match Ini::load_from_file("RenegadeX-Launcher.ini") {
    Ok(conf) => conf,
    Err(_e) => {
      //TODO spawn dialog that gets PlayerName
      let mut conf = Ini::new();
      conf.with_section(Some("RenX_Launcher"))
        .set("GameLocation", "C:/Program Files (x86)/Renegade X/")
        .set("VersionUrl", "https://static.renegade-x.com/launcher_data/version/release.json")
        .set("PlayerName", "")
        .set("LauncherTheme", "dom")
        .set("LastNewsGUID", "")
        .set("64-bit-version", "false");
      conf.write_to_file("RenegadeX-Launcher.ini").unwrap();
      conf
    }
  };

  let section = conf.section(Some("RenX_Launcher".to_owned())).unwrap();
  let game_location = section.get("GameLocation").unwrap();
  let version_url = section.get("VersionUrl").unwrap();
  let launcher_theme = section.get("LauncherTheme").unwrap();
  let playername = section.get("PlayerName").unwrap().clone();

  let mut current_path = std::env::current_exe().unwrap();
  current_path.pop();
  sciter::set_options(
    sciter::RuntimeOptions::ScriptFeatures(
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8 | // Enables Sciter.machineName()
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO as u8 | // Enables opening file dialog (view.selectFile())
      sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SOCKET_IO as u8
    )
  ).unwrap(); // Enables connecting to the inspector via Ctrl+Shift+I
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
  current_path.push(format!("{}/load-page.htm", launcher_theme));
  frame.load_file(current_path.to_str().unwrap());

  let irc_thread = std::thread::spawn(move || {
    let config = Config {
      nickname: Some(playername.to_owned()),
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
        println!("{:#?}", &channel);
        if channel == "#renegadex" {
          let mut ui_option = irc_callback.lock().unwrap();
          match *ui_option {
            Some(ref ui) => {
              ui.call(None, &make_args!(message.as_str()), None).unwrap();
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
