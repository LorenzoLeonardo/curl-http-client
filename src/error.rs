use std::fmt::Debug;

use crate::ExtendedHandler;

/// Error type returned by failed curl HTTP requests.
#[derive(Debug)]
pub enum Error<C>
where
    C: ExtendedHandler + Debug + Send + 'static,
{
    Curl(curl::Error),
    Http(String),
    Perform(async_curl::error::Error<C>),
    Other(String),
}

impl<C> std::fmt::Display for Error<C>
where
    C: ExtendedHandler + Debug + Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Curl(err) => write!(f, "{}", err),
            Error::Http(err) => write!(f, "{}", err),
            Error::Perform(err) => write!(f, "{}", err),
            Error::Other(err) => write!(f, "{}", err),
        }
    }
}

impl<C> std::error::Error for Error<C> where C: ExtendedHandler + Debug + Send + 'static {}
