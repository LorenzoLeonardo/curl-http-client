use std::fs;
use std::path::PathBuf;

use async_curl::async_curl::AsyncCurl;
use curl_http_client::{
    collector::{Collector, FileInfo},
    http_client::{BytesOffset, HttpClient},
    request::HttpRequest,
};
use http::{HeaderMap, Method};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let curl = AsyncCurl::new();
    let save_to = PathBuf::from("<FILE PATH TO SAVE>");
    let collector = Collector::File(FileInfo::path(save_to.clone()));

    let partial_download_file_size = fs::metadata(save_to.as_path())?.len() as usize;
    let request = HttpRequest {
        url: Url::parse("<SOURCE URL>")?,
        method: Method::GET,
        headers: HeaderMap::new(),
        body: None,
    };

    let response = HttpClient::new(curl, collector)
        .resume_from(BytesOffset::from(partial_download_file_size))?
        .request(request)?
        .perform()
        .await?;

    println!("Response: {:?}", response);
    Ok(())
}
