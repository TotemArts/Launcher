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
mod spawn_wrapper;
mod version_information;
mod as_string;
mod functions;

use crate::error::Error;
use flexi_logger::{Age, Criterion, Cleanup, Logger, Naming};
use log::*;
use single_instance::SingleInstance;
use tokio::sync::Mutex;
use std::sync::Arc;
use handler::Handler;

static VERSION: &str = env!("CARGO_PKG_VERSION");

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

#[derive(Default)]
struct DebugHandler;

impl sciter::HostHandler for DebugHandler {
  fn on_debug_output(&mut self, subsystem: sciter::host::OUTPUT_SUBSYTEMS, severity: sciter::host::OUTPUT_SEVERITY, message: &str) {
    let severity = match severity {
      sciter::host::OUTPUT_SEVERITY::INFO => Level::Info,
      sciter::host::OUTPUT_SEVERITY::WARNING => Level::Warn,
      sciter::host::OUTPUT_SEVERITY::ERROR => Level::Error
    };

    log::logger().log(&Record::builder()
    .args(format_args!("{}", message))
    .level(severity)
    .file(Some(&format!("sciter:{:?}", subsystem)))
    .module_path(None)
    .build());
  }

  fn on_engine_destroyed(&mut self) {
    info!("Sciter Engine shutting down!");
    log::logger().flush();
  }
}

fn main() -> Result<(), Error> {
  let current_dir = std::env::current_dir()?;
  info!("Working in directory: {}", &current_dir.to_string_lossy());
  const WEBIFY: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC.remove(b'/').remove(b'\\').remove(b':');
  let current_dir = percent_encoding::utf8_percent_encode(current_dir.to_str().expect(concat!(file!(),":",line!())), WEBIFY).to_string();

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

  let instance = SingleInstance::new("RenegadeX-Launcher").unwrap();
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

  Logger::try_with_env_or_str("info").unwrap()
    .format(flexi_logger::opt_format)
    .log_to_file(flexi_logger::FileSpec::default().directory(&log_directory))
    .rotate(Criterion::Age(Age::Day), Naming::Numbers, Cleanup::KeepLogFiles(5))
    .print_message()
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

  let unwind = std::panic::catch_unwind(|| {
    launch_ui(current_dir).join().map_err(|e| Error::None(format!("{:?}", e)))??;
    Ok::<(), Error>(())
  });

  if let Err(e) = unwind {
    error!("sciter panicked with: {:?}", e);
  } else if let Ok(Err(e)) = unwind {
    error!("sciter panicked with: {:?}", e);
  }


  log::logger().flush();
  Ok(())
}

fn launch_ui(current_dir: String) -> std::thread::JoinHandle<Result<(),Error>> {
  std::thread::spawn(move || -> Result<(), Error> {
    let configuration = configuration::Configuration::load_or_default();

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
    let runtime : tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("");
    if configuration.get_playername().eq("UnknownPlayer") {
      let mut frame = sciter::Window::new();
      frame.event_handler(Handler{patcher: Arc::new(Mutex::new(None)), version_information: Arc::new(Mutex::new(None)), configuration: configuration.clone(), runtime: runtime.handle().clone()});
      frame.load_file(&format!("file://{}/dom/first-startup.htm", &current_dir));
      frame.run_app();
    }
  
    let launcher_theme = configuration.get_launcher_theme();
    
    info!("Launching sciter!");
  
    let guard = runtime.enter();
    
    let unwind = std::panic::catch_unwind(|| {
      let mut frame = sciter::Window::new();
      frame.sciter_handler(DebugHandler {});
      frame.expand(true);
  
      frame.event_handler(Handler{patcher: Arc::new(Mutex::new(None)), version_information: Arc::new(Mutex::new(None)), configuration, runtime: tokio::runtime::Handle::current()});
      frame.load_file(&format!("file://{}/{}/index.htm", current_dir, &launcher_theme));
      info!("Launching app!");
      frame.run_app();
    });

    if let Err(e) = unwind {
      error!("sciter panicked with: {:?}", e);
    }

    drop(guard);
  
    info!("Gracefully shutting down app!");
    runtime.shutdown_background();
    Ok(())
  })
}
