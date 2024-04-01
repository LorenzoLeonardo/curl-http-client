use std::{fs, path::PathBuf};

use async_curl::CurlActor;
use curl_http_client::*;
use http::{Method, Request};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
