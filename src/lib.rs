//! curl-http-client: This is a wrapper for Easy2 from curl-rust crate for ergonomic use
//! and is able to perform asynchronously using async-curl crate that uses an actor model
//! (Message passing) to achieve a non-blocking I/O.
//! This requires a dependency with the [curl](https://crates.io/crates/curl), [async-curl](https://crates.io/crates/async-curl)
//! [http](https://crates.io/crates/http), [url](https://crates.io/crates/url) and [tokio](https://crates.io/crates/tokio) crates
//!
pub mod collector;
pub mod error;
pub mod http_client;
pub mod request;
pub mod response;
#[cfg(test)]
mod test;
