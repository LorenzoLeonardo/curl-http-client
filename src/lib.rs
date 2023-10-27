//! curl-http-client: This is a wrapper for [Easy2](https://docs.rs/curl/latest/curl/easy/struct.Easy2.html) from [curl-rust](https://docs.rs/curl/latest/curl) crate for ergonomic use
//! and is able to perform asynchronously using [async-curl](https://docs.rs/async-curl/latest/async_curl) crate that uses an actor model
//! (Message passing) to achieve a non-blocking I/O.
//! This requires a dependency with the [curl](https://crates.io/crates/curl), [async-curl](https://crates.io/crates/async-curl)
//! [http](https://crates.io/crates/http), [url](https://crates.io/crates/url) and [tokio](https://crates.io/crates/tokio) crates
//!
//! # Get Request
//! ```rust,no_run
//! use async_curl::async_curl::AsyncCurl;
//! use curl_http_client::{collector::Collector, http_client::HttpClient, request::HttpRequest};
//! use http::{HeaderMap, Method};
//! use url::Url;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let curl = AsyncCurl::new();
//!     let collector = Collector::Ram(Vec::new());
//!
//!     let request = HttpRequest {
//!         url: Url::parse("<SOURCE URL>")?,
//!         method: Method::GET,
//!         headers: HeaderMap::new(),
//!         body: None,
//!     };
//!
//!     let response = HttpClient::new(curl, collector)
//!         .request(request)?
//!         .perform()
//!         .await?;
//!
//!     println!("Response: {:?}", response);
//!     Ok(())
//! }
//! ```
//!
//! # Post Request
//! ```rust,no_run
//! use async_curl::async_curl::AsyncCurl;
//! use curl_http_client::{collector::Collector, http_client::HttpClient, request::HttpRequest};
//! use http::{HeaderMap, Method};
//! use url::Url;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let curl = AsyncCurl::new();
//!     let collector = Collector::Ram(Vec::new());
//!
//!     let request = HttpRequest {
//!         url: Url::parse("<TARGET URL>")?,
//!         method: Method::POST,
//!         headers: HeaderMap::new(),
//!         body: Some("test body".as_bytes().to_vec()),
//!     };
//!
//!     let response = HttpClient::new(curl, collector)
//!         .request(request)?
//!         .perform()
//!         .await?;
//!
//!     println!("Response: {:?}", response);
//!     Ok(())
//! }
//! ```
//!
//! # Downloading a File
//! ```rust,no_run
//! use std::path::PathBuf;
//!
//! use async_curl::async_curl::AsyncCurl;
//! use curl_http_client::{
//!     collector::{Collector, FileInfo},
//!     http_client::HttpClient,
//!     request::HttpRequest,
//! };
//! use http::{HeaderMap, Method};
//! use url::Url;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let curl = AsyncCurl::new();
//!
//!     let collector = Collector::File(FileInfo::path(PathBuf::from("<FILE PATH TO SAVE>")));
//!
//!     let request = HttpRequest {
//!         url: Url::parse("<SOURCE URL>")?,
//!         method: Method::GET,
//!         headers: HeaderMap::new(),
//!         body: None,
//!     };
//!
//!     let response = HttpClient::new(curl, collector)
//!         .request(request)?
//!         .perform()
//!         .await?;
//!
//!     println!("Response: {:?}", response);
//!     Ok(())
//! }
//! ```
//!
//! # Uploading a File
//! ```rust,no_run
//! use std::{fs, path::PathBuf};
//!
//! use async_curl::async_curl::AsyncCurl;
//! use curl_http_client::{
//!     collector::{Collector, FileInfo},
//!     http_client::{FileSize, HttpClient},
//!     request::HttpRequest,
//! };
//! use http::{HeaderMap, Method};
//! use url::Url;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let file_to_be_uploaded = PathBuf::from("<FILE PATH TO BE UPLOADED>");
//!     let file_size = fs::metadata(file_to_be_uploaded.as_path()).unwrap().len() as usize;
//!
//!     let curl = AsyncCurl::new();
//!     let collector = Collector::File(FileInfo::path(file_to_be_uploaded));
//!
//!     let request = HttpRequest {
//!         url: Url::parse("<TARGET URL>")?,
//!         method: Method::PUT,
//!         headers: HeaderMap::new(),
//!         body: None,
//!     };
//!
//!     let response = HttpClient::new(curl, collector)
//!         .upload_file_size(FileSize::from(file_size))?
//!         .request(request)?
//!         .perform()
//!         .await?;
//!
//!     println!("Response: {:?}", response);
//!     Ok(())
//! }
//! ```
//!
//! # Concurrency
//! ```rust,no_run
//! use async_curl::async_curl::AsyncCurl;
//! use curl_http_client::{collector::Collector, http_client::HttpClient, request::HttpRequest};
//! use futures::future;
//! use http::{HeaderMap, Method};
//! use url::Url;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     const NUM_CONCURRENT: usize = 5;
//!
//!     let curl = AsyncCurl::new();
//!     let mut handles = Vec::new();
//!
//!     for _n in 0..NUM_CONCURRENT {
//!         let curl = curl.clone();
//!
//!         let handle = tokio::spawn(async move {
//!             let collector = Collector::Ram(Vec::new());
//!             let request = HttpRequest {
//!                 url: Url::parse("https://www.rust-lang.org/").unwrap(),
//!                 method: Method::GET,
//!                 headers: HeaderMap::new(),
//!                 body: None,
//!             };
//!
//!             let response = HttpClient::new(curl, collector)
//!                 .request(request)
//!                 .unwrap()
//!                 .perform()
//!                 .await
//!                 .unwrap();
//!             println!("Response: {:?}", response);
//!         });
//!         handles.push(handle);
//!     }
//!
//!     let results: Vec<Result<_, _>> = future::join_all(handles).await;
//!
//!     for (_i, result) in results.into_iter().enumerate() {
//!         result.unwrap();
//!     }
//! }
//! ```
//!
//! # Resume Downloading a File
//! ```rust,no_run
//! use std::fs;
//! use std::path::PathBuf;
//!
//! use async_curl::async_curl::AsyncCurl;
//! use curl_http_client::{
//!     collector::{Collector, FileInfo},
//!     http_client::{BytesOffset, HttpClient},
//!     request::HttpRequest,
//! };
//! use http::{HeaderMap, Method};
//! use url::Url;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let curl = AsyncCurl::new();
//!     let save_to = PathBuf::from("<FILE PATH TO SAVE>");
//!     let collector = Collector::File(FileInfo::path(save_to.clone()));
//!
//!     let partial_download_file_size = fs::metadata(save_to.as_path())?.len() as usize;
//!     let request = HttpRequest {
//!         url: Url::parse("<SOURCE URL>")?,
//!         method: Method::GET,
//!         headers: HeaderMap::new(),
//!         body: None,
//!     };
//!
//!     let response = HttpClient::new(curl, collector)
//!         .resume_from(BytesOffset::from(partial_download_file_size))?
//!         .request(request)?
//!         .perform()
//!         .await?;
//!
//!     println!("Response: {:?}", response);
//!     Ok(())
//! }
//! ```
//!
pub mod collector;
pub mod error;
pub mod http_client;
pub mod request;
pub mod response;
#[cfg(test)]
mod test;
