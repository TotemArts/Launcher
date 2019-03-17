#[cfg(windows)]
extern crate winres;
#[cfg(windows)]
use std::io::Write;

#[cfg(windows)]
fn main() {
  let target_os = match std::env::var("CARGO_CFG_TARGET_OS") {
    Ok(value) => value,
    Err(e) =>  panic!("No environmental variable found that matches CARGO_CFG_TARGET_OS: {}", e)
  };
  if target_os == "windows" {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("manifest.xml");
    res.set_icon("rx.ico");
    match res.compile() {
      Err(e) => {
        write!(std::io::stderr(), "windows.rs: 15 -> {}", e).unwrap();
        std::process::exit(1);
      },
      Ok(_) => {}
    }
  }
}

#[cfg(unix)]
fn main() {
}
