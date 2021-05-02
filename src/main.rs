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
mod handler;
mod progress;
mod error;

use flexi_logger::{Age, Criterion, Cleanup, Logger, Naming};
use log::*;

use single_instance::SingleInstance;

use std::sync::{Arc,Mutex};
use renegadex_patcher::{Downloader};
use handler::Handler;

static VERSION: &str = env!("CARGO_PKG_VERSION");


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
  let mut current_dir = std::env::current_exe()?;
  current_dir.pop();
  info!("Working in directory: {}", &current_dir.to_string_lossy());
  std::env::set_current_dir(&current_dir)?;
  const WEBIFY: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC.remove(b'/').remove(b'\\').remove(b':');
  let current_dir = percent_encoding::utf8_percent_encode(current_dir.to_str().unexpected(concat!(file!(),":",line!())), WEBIFY).to_string();

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

  let instance = SingleInstance::new("RenegadeX-Launcher")?;
  //TODO: Create "Another instance is already running" window.
  if !instance.is_single() {
    let mut frame = sciter::Window::new();
    frame.event_handler(UpdateResultHandler{update_result: "".to_owned()});
    frame.load_file(&format!("file://{}/dom/instance.htm", &current_dir));
    frame.run_app();
    std::process::exit(0);
  }

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

  std::panic::set_hook(Box::new(|panic_info| {
    log::logger().log(&Record::builder()
    .args(format_args!("{}", panic_info.to_string()))
    .level(Level::Error)
    .file(panic_info.location().map(|a| a.file()))
    .line(panic_info.location().map(|a| a.line()))
    .module_path(None)
    .build());
  }));

  info!("Starting RenegadeX Launcher version {}", &VERSION);

  for argument in std::env::args() {
    if argument.starts_with("--patch-result=") {
      info!("Update result: {}", &argument[15..].to_string());
      let mut frame = sciter::Window::new();
      frame.event_handler(UpdateResultHandler{update_result: argument[15..].to_string()});
      frame.load_file(&format!("file://{}/dom/self-update-result.htm", &current_dir));
      frame.run_app();
    }
  }

  let patcher : Arc<Mutex<Downloader>> = Arc::new(Mutex::new(Downloader::new()));
  if configuration.get_playername().eq("UnknownPlayer") {
    let mut frame = sciter::Window::new();
    frame.event_handler(Handler{patcher: patcher.clone(), configuration: configuration.clone(), runtime: tokio::runtime::Handle::current()});
    frame.load_file(&format!("file://{}/dom/first-startup.htm", &current_dir));
    frame.run_app();
  }

  let game_location = configuration.get_game_location();
  let version_url = configuration.get_version_url();
  let launcher_theme = configuration.get_launcher_theme();
  
  info!("Launching sciter!");

  let mut frame = sciter::Window::new();
  let mut locked_patcher = patcher.lock().unwrap();
  locked_patcher.set_location(game_location);
  locked_patcher.set_version_url(version_url);
  drop(locked_patcher);
  
  frame.event_handler(Handler{patcher: patcher.clone(), configuration, runtime: tokio::runtime::Handle::current()});
  frame.load_file(&format!("file://{}/{}/frontpage.htm", current_dir, &launcher_theme));
  info!("Launching app!");

  frame.run_app();

  info!("Gracefully shutting down app!");
  log::logger().flush();
  Ok(())
}
