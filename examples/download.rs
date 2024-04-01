use std::path::PathBuf;

use async_curl::CurlActor;
use curl_http_client::*;
use http::{Method, Request};

#[tokio::main(flavor = "current_thread")]
async fn main() {
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
        .await
        .unwrap();

    println!("Response: {:?}", response);
}
