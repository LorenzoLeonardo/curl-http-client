# curl-http-client

This is a wrapper for Easy2 from curl-rust crate for ergonomic use
and is able to perform synchronously and asynchronously using async-curl crate
that uses an actor model (Message passing) to achieve a non-blocking I/O.

[![Latest Version](https://img.shields.io/crates/v/curl-http-client.svg)](https://crates.io/crates/curl-http-client)
[![License](https://img.shields.io/github/license/LorenzoLeonardo/curl-http-client.svg)](LICENSE-MIT)
[![Documentation](https://docs.rs/curl-http-client/badge.svg)](https://docs.rs/curl-http-client)
[![Build Status](https://github.com/LorenzoLeonardo/curl-http-client/workflows/Rust/badge.svg)](https://github.com/LorenzoLeonardo/curl-http-client/actions)

# Asynchronous Examples

## Get Request
```rust
use async_curl::actor::CurlActor;
use curl_http_client::{collector::Collector, http_client::HttpClient};
use http::{Method, Request};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());

    let request = Request::builder()
        .uri("<SOURCE URL>")
        .method(Method::GET)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request).unwrap()
        .nonblocking(actor)
        .perform()
        .await.unwrap();
    println!("Response: {:?}", response);
}
```

## Post Request
```rust
use async_curl::actor::CurlActor;
use curl_http_client::{collector::Collector, http_client::HttpClient};
use http::{Method, Request};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());

    let request = Request::builder()
        .uri("<TARGET URL>")
        .method(Method::POST)
        .body(Some("test body".as_bytes().to_vec()))
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request).unwrap()
        .nonblocking(actor)
        .perform()
        .await.unwrap();

    println!("Response: {:?}", response);
}
```

## Downloading a File
```rust
use std::path::PathBuf;

use async_curl::actor::CurlActor;
use curl_http_client::{
    collector::{Collector, FileInfo},
    http_client::HttpClient,
};
use http::{Method, Request};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let actor = CurlActor::new();

    let collector = Collector::File(FileInfo::path(PathBuf::from("<FILE PATH TO SAVE>")));

    let request = Request::builder()
        .uri("<SOURCE URL>")
        .method(Method::GET)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .nonblocking(actor)
        .perform()
        .await.unwrap();

    println!("Response: {:?}", response);
    Ok(())
}
```

## Uploading a File
```rust
use std::{fs, path::PathBuf};

use async_curl::actor::CurlActor;
use curl_http_client::{
    collector::{Collector, FileInfo},
    http_client::{FileSize, HttpClient},
};
use http::{HeaderMap, Method, Request};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let file_to_be_uploaded = PathBuf::from("<FILE PATH TO BE UPLOADED>");
    let file_size = fs::metadata(file_to_be_uploaded.as_path()).unwrap().len() as usize;

    let actor = CurlActor::new();
    let collector = Collector::File(FileInfo::path(file_to_be_uploaded));

    let request = Request::builder()
        .uri("<TARGET URL>")
        .method(Method::PUT)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .upload_file_size(FileSize::from(file_size)).unwrap()
        .request(request).unwrap()
        .nonblocking(actor)
        .perform()
        .await.unwrap();

    println!("Response: {:?}", response);
}
```

## Concurrency
```rust
use async_curl::actor::CurlActor;
use curl_http_client::{collector::Collector, http_client::HttpClient};
use futures::future;
use http::{HeaderMap, Method, Request};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    const NUM_CONCURRENT: usize = 5;

    let actor = CurlActor::new();
    let mut handles = Vec::new();

    for _n in 0..NUM_CONCURRENT {
        let actor = actor.clone();

        let handle = tokio::spawn(async move {
            let collector = Collector::Ram(Vec::new());
            let request = Request::builder()
                .uri("https://www.rust-lang.org/")
                .method(Method::GET)
                .body(None)
                .unwrap();

            let response = HttpClient::new(collector)
                .request(request)
                .unwrap()
                .nonblocking(actor)
                .perform()
                .await
                .unwrap();
            println!("Response: {:?}", response);
        });
        handles.push(handle);
    }

    let results: Vec<Result<_, _>> = future::join_all(handles).await;

    for (_i, result) in results.into_iter().enumerate() {
        result.unwrap();
    }
}
```

## Resume Downloading a File
```rust
use std::fs;
use std::path::PathBuf;

use async_curl::actor::CurlActor;
use curl_http_client::{
    collector::{Collector, FileInfo},
    http_client::{BytesOffset, HttpClient},
};
use http::{HeaderMap, Method, Request};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let actor = CurlActor::new();
    let save_to = PathBuf::from("<FILE PATH TO SAVE>");
    let collector = Collector::File(FileInfo::path(save_to.clone()));

    let partial_download_file_size = fs::metadata(save_to.as_path()).unwrap().len() as usize;
    let request = Request::builder()
        .uri("<SOURCE URL>")
        .method(Method::GET)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .resume_from(BytesOffset::from(partial_download_file_size)).unwrap()
        .request(request).unwrap()
        .nonblocking(actor)
        .perform()
        .await.unwrap();

    println!("Response: {:?}", response);
}
```

## Downloading a File with download speed information sent to different task
```rust
use std::path::PathBuf;

use async_curl::actor::CurlActor;
use curl_http_client::{
    collector::{Collector, FileInfo},
    http_client::HttpClient,
};
use http::{HeaderMap, Method, Request};
use tokio::sync::mpsc::channel;
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (tx, mut rx) = channel(1);

    let actor = CurlActor::new();
    let file_info = FileInfo::path(PathBuf::from("<FILE PATH TO SAVE>")).with_transfer_speed_sender(tx);
    let collector = Collector::File(file_info);

    let handle = tokio::spawn(async move {
        while let Some(speed) = rx.recv().await {
            println!("Download Speed: {} kB/s", speed.as_bytes_per_sec());
        }
    });

    let request = Request::builder()
        .uri("<SOURCE URL>")
        .method(Method::GET)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request).unwrap()
        .nonblocking(actor)
        .perform()
        .await.unwrap();

    println!("Response: {:?}", response);

    handle.abort();
}
```

## Uploading a File with upload speed information sent to different task
```rust
use std::{fs, path::PathBuf};

use async_curl::actor::CurlActor;
use curl_http_client::{
    collector::{Collector, FileInfo},
    http_client::{FileSize, HttpClient},
};
use http::{HeaderMap, Method, Request};
use tokio::sync::mpsc::channel;
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (tx, mut rx) = channel(1);

    let file_to_be_uploaded = PathBuf::from("<FILE PATH TO BE UPLOADED>");
    let file_size = fs::metadata(file_to_be_uploaded.as_path()).unwrap().len() as usize;

    let actor = CurlActor::new();
    let file_info = FileInfo::path(file_to_be_uploaded).with_transfer_speed_sender(tx);
    let collector = Collector::File(file_info);

    let handle = tokio::spawn(async move {
        while let Some(speed) = rx.recv().await {
            println!("Upload Speed: {} kB/s", speed.as_bytes_per_sec());
        }
    });

    let request = Request::builder()
        .uri("<TARGET URL>")
        .method(Method::PUT)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .upload_file_size(FileSize::from(file_size)).unwrap()
        .request(request).unwrap()
        .nonblocking(actor)
        .perform()
        .await.unwrap();

    println!("Response: {:?}", response);
    handle.abort();
}
```

# Synchronous Examples

## Get Request
```rust
use curl_http_client::{collector::Collector, http_client::HttpClient};
use http::{HeaderMap, Method, Request};
use url::Url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let collector = Collector::Ram(Vec::new());

let request = Request::builder()
    .uri("<SOURCE URL>")
    .method(Method::GET)
    .body(None)
    .unwrap();

    let response = HttpClient::new(collector)
        .request(request)?
        .blocking()
        .perform()?;

    println!("Response: {:?}", response);
    Ok(())
}
```

## Post Request
```rust
use curl_http_client::{collector::Collector, http_client::HttpClient};
use http::{HeaderMap, Method, Request};
use url::Url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let collector = Collector::Ram(Vec::new());

let request = Request::builder()
    .uri("<TARGET URL>")
    .method(Method::POST)
    .body(Some("test body".as_bytes().to_vec()))
    .unwrap();

    let response = HttpClient::new(collector)
        .request(request)?
        .blocking()
        .perform()?;

    println!("Response: {:?}", response);
    Ok(())
}
```

## Downloading a File
```rust
use std::path::PathBuf;

use curl_http_client::{
    collector::{Collector, FileInfo},
    http_client::HttpClient,
};
use http::{HeaderMap, Method, Request};
use url::Url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let collector = Collector::File(FileInfo::path(PathBuf::from("<FILE PATH TO SAVE>")));

let request = Request::builder()
    .uri("<SOURCE URL>")
    .method(Method::GET)
    .body(None)
    .unwrap();

    let response = HttpClient::new(collector)
        .request(request)?
        .blocking()
        .perform()?;

    println!("Response: {:?}", response);
    Ok(())
}
```

## Uploading a File
```rust
use std::{fs, path::PathBuf};

use curl_http_client::{
    collector::{Collector, FileInfo},
    http_client::{FileSize, HttpClient},
};
use http::{HeaderMap, Method, Request};
use url::Url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_to_be_uploaded = PathBuf::from("<FILE PATH TO BE UPLOADED>");
    let file_size = fs::metadata(file_to_be_uploaded.as_path()).unwrap().len() as usize;

    let collector = Collector::File(FileInfo::path(file_to_be_uploaded));

let request = Request::builder()
    .uri("<TARGET URL>")
    .method(Method::PUT)
    .body(None)
    .unwrap();

    let response = HttpClient::new(collector)
        .upload_file_size(FileSize::from(file_size))?
        .request(request)?
        .blocking()
        .perform()?;

    println!("Response: {:?}", response);
    Ok(())
}
```
