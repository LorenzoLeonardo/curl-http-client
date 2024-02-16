use std::{fs, path::PathBuf};

use async_curl::actor::CurlActor;
use curl_http_client::{
    collector::{Collector, FileInfo},
    http_client::{FileSize, HttpClient},
    request::HttpRequest,
};
use http::{HeaderMap, Method};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_to_be_uploaded = PathBuf::from("<FILE PATH TO BE UPLOADED>");
    let file_size = fs::metadata(file_to_be_uploaded.as_path()).unwrap().len() as usize;

    let actor = CurlActor::new();
    let collector = Collector::File(FileInfo::path(file_to_be_uploaded));

    let request = HttpRequest {
        url: Url::parse("<TARGET URL>").unwrap(),
        method: Method::PUT,
        headers: HeaderMap::new(),
        body: None,
    };

    let response = HttpClient::new(collector)
        .upload_file_size(FileSize::from(file_size))
        .unwrap()
        .request(request)
        .unwrap()
        .nonblocking(actor)
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);
    Ok(())
}
