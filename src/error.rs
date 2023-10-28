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
