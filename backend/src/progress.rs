use async_trait::async_trait;
use sciter::Value;
use crate::error::Error;
use log::error;

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
  async fn set_file_size(&mut self, size: usize) {
    self.file_size = size;

    let file_size = self.file_size;
    let downloaded = 0;
    let progress = self.progress.lock().unwrap().clone();
    crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
      progress.call(None, &make_args!(format!("[{}, {}]", downloaded, file_size)), None).or_else(|e| Err(Error::ValueError(e)))?;
      Ok(())
    });
  }

  async fn add_to_progress(&mut self, amount: usize) {
    self.downloaded += amount;
    let file_size = self.file_size;
    let downloaded = self.downloaded;
    let progress = self.progress.lock().unwrap().clone();
    crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
      progress.call(None, &make_args!(format!("[{}, {}]", downloaded, file_size)), None).or_else(|e| Err(Error::ValueError(e)))?;
      Ok(())
    });
  }

  async fn remove_from_progress(&mut self, amount: usize) {
    self.downloaded -= amount;
    let file_size = self.file_size;
    let downloaded = self.downloaded;
    let progress = self.progress.lock().unwrap().clone();
    crate::spawn_wrapper::spawn(move || -> Result<(), Error> {
      progress.call(None, &make_args!(format!("[{}, {}]", downloaded, file_size)), None).or_else(|e| Err(Error::ValueError(e)))?;
      Ok(())
    });
  }
}

/// Convert a raw bytesize into a network speed
pub fn convert(num: f64) -> String {
  let negative = if num.is_sign_positive() { "" } else { "-" };
  let num = num.abs();
  let units = ["B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
  if num < 1_f64 {
    return format!("{}{} {}", negative, num, "B");
  }
  let delimiter = 1000_f64;
  let exponent = std::cmp::min((num.ln() / delimiter.ln()).floor() as i32, (units.len() - 1) as i32);
  let pretty_bytes = format!("{:.2}", num / delimiter.powi(exponent)).parse::<f64>().unwrap_or_else(|error| {
    error!("{}:{}:{} has encountered an parsing issue: {}", module_path!(),file!(),line!(), error);
    panic!("{}:{}:{} has encountered an parsing issue: {}", module_path!(),file!(),line!(), error)
  }) * 1_f64;
  let unit = units[exponent as usize];
  format!("{}{} {}", negative, pretty_bytes, unit)
}