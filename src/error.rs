use thiserror::Error;
use workflow_core::channel::{RecvError, SendError, TryRecvError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Already running")]
    AlreadyRunning,
    #[error("The task is not running")]
    NotRunning,
    #[error("{0:?}")]
    SendError(String),
    #[error("{0:?}")]
    RecvError(#[from] RecvError),
    #[error("{0:?}")]
    TryRecvError(#[from] TryRecvError),
    #[error(transparent)]
    TaskError(#[from] workflow_core::task::TaskError),
    #[error(transparent)]
    CallbackError(#[from] workflow_wasm::callback::CallbackError),
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl<T> From<SendError<T>> for Error {
    fn from(err: SendError<T>) -> Self {
        Error::SendError(err.to_string())
    }
}
