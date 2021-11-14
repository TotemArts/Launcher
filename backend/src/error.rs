use sciter::value::VALUE_RESULT;
use log::{Record, Level};

#[derive(Debug)]
pub enum Error {
  InvalidUri(download_async::http::uri::InvalidUri),
	MutexPoisoned(String),
	None(String),
  IoError(std::io::Error),
	DownloadError(Box<dyn std::error::Error + Sync + std::marker::Send>),
	DownloadAsyncError(download_async::Error),
  PatcherError(renegadex_patcher::Error),
  ValueError(VALUE_RESULT),
  NotUtf8(std::string::FromUtf8Error),
  Utf8Error(std::str::Utf8Error),
  ParseError(url::ParseError),
  ParseIntError(std::num::ParseIntError),
  UnzipError(std::io::Error),
}

impl std::error::Error for Error { }

impl std::fmt::Display for Error {
  #[track_caller]
  #[inline(always)]
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::InvalidUri(e) => write!(f,"InvalidUri({:?})", e),
      Self::MutexPoisoned(e) => write!(f,"MutexPoisoned({:?})", e),
      Self::None(e) => write!(f,"None({:?})", e),
      Self::IoError(e) => write!(f,"IoError({:?})", e),
      Self::DownloadError(e) => write!(f,"DownloadError({:?})", e),
      Self::DownloadAsyncError(e) => write!(f,"DownloadAsyncError({:?})", e),
      Self::PatcherError(e) => write!(f,"PatcherError({:?})", e),
      Self::ValueError(e) => write!(f,"ValueError({:?})", e),
      Self::NotUtf8(e) => write!(f,"NotUtf8({:?})", e),
      Self::Utf8Error(e) => write!(f,"Utf8Error({:?})", e),
      Self::ParseError(e) => write!(f,"ParseError({:?})", e),
      Self::ParseIntError(e) => write!(f,"ParseIntError({:?})", e),
      Self::UnzipError(e) => write!(f,"UnzipError({:?})", e),
    }
  }
}

impl<T> From<std::sync::PoisonError<std::sync::MutexGuard<'_, T>>> for Error {
  #[track_caller]
  #[inline(always)]
  fn from(error: std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
    log_error(&error);
    use std::error::Error;
    let error = error.source().unwrap();
    log_error(error);
    Self::MutexPoisoned(error.to_string())
  }
}

impl From<Box<dyn std::error::Error + Sync + std::marker::Send>> for Error {
  #[track_caller]
  #[inline(always)]
  fn from(error: Box<dyn std::error::Error + Sync + std::marker::Send>) -> Self {
    log_error(error.source().unwrap());
    Self::DownloadError(error)
  }
}

impl From<download_async::Error> for Error {
  #[track_caller]
  #[inline(always)]
  fn from(error: download_async::Error) -> Self {
    log_error(&error);
    Self::DownloadAsyncError(error)
  }
}

impl From<renegadex_patcher::Error> for Error {
  #[track_caller]
  #[inline(always)]
  fn from(error: renegadex_patcher::Error) -> Self {
    log_error(&error);
    Self::PatcherError(error)
  }
}

impl From<sciter::value::VALUE_RESULT> for Error {
  #[track_caller]
  #[inline(always)]
  fn from(error: sciter::value::VALUE_RESULT) -> Self {
    log_error(&error);
    Self::ValueError(error)
  }
}

impl From<std::io::Error> for Error {
  #[track_caller]
  #[inline(always)]
  fn from(error: std::io::Error) -> Self {
    log_error(&error);
    Self::IoError(error)
  }
}

impl From<std::string::FromUtf8Error> for Error {
  #[track_caller]
  #[inline(always)]
  fn from(error: std::string::FromUtf8Error) -> Self {
    log_error(&error);
    Self::NotUtf8(error)
  }
}

impl From<download_async::http::uri::InvalidUri> for Error {
  #[track_caller]
  #[inline(always)]
  fn from(error: download_async::http::uri::InvalidUri) -> Self {
    log_error(&error);
    Self::InvalidUri(error)
  }
}

impl From<url::ParseError> for Error {
  #[track_caller]
  #[inline(always)]
  fn from(error: url::ParseError) -> Self {
    log_error(&error);
    Self::ParseError(error)
  }
}

impl From<std::num::ParseIntError> for Error {
  #[track_caller]
  #[inline(always)]
  fn from(error: std::num::ParseIntError) -> Self {
    log_error(&error);
    Self::ParseIntError(error)
  }
}

impl From<std::str::Utf8Error> for Error {
  #[track_caller]
  #[inline(always)]
  fn from(error: std::str::Utf8Error) -> Self {
    log_error(&error);
    Self::Utf8Error(error)
  }
}

#[track_caller]
fn log_error(error: &(impl std::error::Error + ?Sized)) {
  let location = Some(std::panic::Location::caller());
  log::logger().log(&Record::builder()
  .args(format_args!("{:?}", error))
  .level(Level::Error)
  .file(location.map(|a| a.file()))
  .line(location.map(|a| a.line()))
  .module_path(None)
  .build());
  log::logger().flush();
}