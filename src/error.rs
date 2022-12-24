use workflow_core::channel::*;
use thiserror::Error;

#[derive(Debug,Error)]
pub enum Error {
    #[error("{0:?}")]
    SendError(String),
    #[error("{0:?}")]
    RecvError(#[from] RecvError),
    #[error("{0:?}")]
    TryRecvError(#[from] TryRecvError),
}

impl<T> From<SendError<T>> for Error {
    fn from(err: SendError<T>) -> Self {
        Error::SendError(err.to_string())
    }
}
