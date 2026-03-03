use std::io;

#[derive(Debug, thiserror::Error)]
pub enum SigocrError {
    #[error("ENOENT: {0}")]
    NotFound(String),

    #[error("{0}")]
    Io(#[from] io::Error),

    #[error("PDF error: {0}")]
    Pdf(String),

    #[error("{0}")]
    Other(String),
}

impl From<SigocrError> for napi::Error {
    fn from(err: SigocrError) -> Self {
        napi::Error::new(napi::Status::GenericFailure, err.to_string())
    }
}

/// Run a blocking closure on the tokio thread pool and map join errors
/// to NAPI errors. Use this for all async NAPI exports that wrap sync logic.
pub async fn spawn_blocking_napi<F, T>(f: F) -> napi::Result<T>
where
    F: FnOnce() -> napi::Result<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f).await.map_err(|e| {
        napi::Error::new(
            napi::Status::GenericFailure,
            format!("Task join error: {e}"),
        )
    })?
}
