use prost::{EncodeError, DecodeError};
use yamux::ConnectionError;



pub type Result<T> = std::result::Result<T, KvError>;

#[derive(thiserror::Error, Debug)]
pub enum KvError {
    #[error("not found table {0} or key {1}")]
    NotFound(String, String),
    #[error("{0}")]
    Invalid(String),
    #[error("invalid command {0}")]
    InvalidCommand(String),
    #[error("{0}")]
    Internal(String),

    #[error("failed to convert value")]
    ConvertError,

    #[error(transparent)]
    SledError(#[from] sled::Error),

    #[error("frame error")]
    FrameError,
    #[error("frame encode error")]
    FrameEncodeError(#[from] EncodeError),
    #[error("frame decode error")]
    FrameDecodeError(#[from] DecodeError),
    #[error("io error")]
    IOError(#[from] std::io::Error),

    #[error("connection error")]
    ConnectionError(#[from] ConnectionError),

    #[error("unknown error")]
    Unknown
}