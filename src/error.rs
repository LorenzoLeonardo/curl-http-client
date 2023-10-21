///
/// Error type returned by failed curl HTTP requests.
///
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("curl request failed")]
    Curl(String),
    #[error("Other error: {}", _0)]
    Other(String),
}
