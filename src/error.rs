use std::fmt::Debug;

use crate::collector::Collector;

/// Error type returned by failed curl HTTP requests.
#[derive(Debug)]
pub enum Error {
    Curl(curl::Error),
    Http(String),
    Perform(async_curl::error::Error<Collector>),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Curl(err) => write!(f, "{}", err),
            Error::Http(err) => write!(f, "{}", err),
            Error::Perform(err) => write!(f, "{}", err),
            Error::Other(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}
