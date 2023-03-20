//!
//! Errors produced by this crate.
//!

use manual_serializer::Error as SerializerError;
use std::string::FromUtf16Error;
use thiserror::Error;

/// Errors produced by this crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Error: {0}")]
    String(String),

    #[error("Error: {0}")]
    FromUtf16Error(FromUtf16Error),

    #[error("{0}")]
    SerializerError(#[from] SerializerError),

    #[error("{0}")]
    Win32Error(::windows::core::Error),
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::String(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Error {
        Error::String(s.to_string())
    }
}

impl From<FromUtf16Error> for Error {
    fn from(e: FromUtf16Error) -> Error {
        Error::FromUtf16Error(e)
    }
}

impl From<::windows::core::Error> for Error {
    fn from(e: ::windows::core::Error) -> Error {
        Error::Win32Error(e)
    }
}
