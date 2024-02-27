use std::fs;
use std::path::PathBuf;

use async_curl::actor::CurlActor;
use curl_http_client::{
    collector::{Collector, FileInfo},
    http_client::{BytesOffset, HttpClient},
};
use http::{Method, Request};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let actor = CurlActor::new();
    let save_to = PathBuf::from("<FILE PATH TO SAVE>");
    let collector = Collector::File(FileInfo::path(save_to.clone()));

    let partial_download_file_size = fs::metadata(save_to.as_path())?.len() as usize;
    let request = Request::builder()
        .uri("<SOURCE URL>")
        .method(Method::GET)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .resume_from(BytesOffset::from(partial_download_file_size))
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
