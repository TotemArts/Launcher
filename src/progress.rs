use async_trait::async_trait;
use sciter::Value;
use crate::error::Error;

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
    std::thread::spawn(move || -> Result<(), Error> {
      progress.call(None, &make_args!(format!("[{}, {}]", downloaded, file_size)), None).or_else(|e| Err(Error::ValueError(e)))?;
      Ok(())
    });
  }

  async fn add_to_progress(&mut self, amount: usize) {
    self.downloaded += amount;
    let file_size = self.file_size;
    let downloaded = self.downloaded;
    let progress = self.progress.lock().unwrap().clone();
    std::thread::spawn(move || -> Result<(), Error> {
      progress.call(None, &make_args!(format!("[{}, {}]", downloaded, file_size)), None).or_else(|e| Err(Error::ValueError(e)))?;
      Ok(())
    });
  }

  async fn remove_from_progress(&mut self, amount: usize) {
    self.downloaded -= amount;
    let file_size = self.file_size;
    let downloaded = self.downloaded;
    let progress = self.progress.lock().unwrap().clone();
    std::thread::spawn(move || -> Result<(), Error> {
      progress.call(None, &make_args!(format!("[{}, {}]", downloaded, file_size)), None).or_else(|e| Err(Error::ValueError(e)))?;
      Ok(())
    });
  }
}