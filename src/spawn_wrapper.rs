use crate::error::Error;


pub fn spawn<F, T>(f: F) -> std::thread::JoinHandle<Result<T,Error>>
where
    F: FnOnce() -> Result<T, Error>,
    F: Send + 'static,
    T: Send + 'static,
{
  let f = move || {
    match f() {
      Ok(t) => Ok(t),
      Err(e) => {
        log::error!("Thread exited with: {}", e);
        Err(e)
      }
    }
  };
  std::thread::spawn(f)
}

pub fn spawn_async<F, R>(handle: &tokio::runtime::Handle, future: F) -> tokio::task::JoinHandle<Result<R,Error>>
where
    F: core::future::Future<Output = Result<R, Error>> + Send + 'static,
    F::Output: Send + 'static,
    R: Send + 'static,
{
  let f = async move {
    match future.await {
      Ok(t) => Ok(t),
      Err(e) => {
        log::error!("Async thread exited with: {}", e);
        Err(e)
      }
    }
  };
  handle.spawn(f)
}