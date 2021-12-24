#![windows_subsystem="windows"]
extern crate sciter;
use std::io::Write;

const SCITER_DLL : &'static [u8] = include_bytes!("../sciter.dll");
const DOM_HTM : &'static [u8] = include_bytes!("dom.htm");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sciter_dll = std::fs::OpenOptions::new().create(true).write(true).open("sciter.dll")?;
    sciter_dll.write_all(SCITER_DLL)?;
    sciter_dll.flush()?;
    drop(sciter_dll);
    let mut frame = sciter::Window::new();
    frame.load_html(DOM_HTM, None);
    frame.run_app();
    Ok(())
}