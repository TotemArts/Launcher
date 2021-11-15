use socket2::*;

use log::*;
use tokio::sync::Mutex;

use std::io::Write;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use renegadex_patcher::{Patcher, PatcherBuilder, Progress};
use sciter::Value;
use sha2::{Sha256, Digest};
use crate::configuration;
use crate::error::Error;
use crate::progress::ValueProgress;
use crate::version_information::VersionInformation;
use std::io::Read;
use ini::Ini;

fn lol(success_callback: Value) {

/*
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
    */
}