use std::path::PathBuf;

use async_curl::async_curl::AsyncCurl;
use curl_http_client::{collector::Collector, http_client::HttpClient, request::HttpRequest};
use http::{HeaderMap, Method};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let curl = AsyncCurl::new();
    let collector = Collector::File(PathBuf::from("<FILE PATH TO SAVE"), 0);

    let request = HttpRequest {
        url: Url::parse("<SOURCE URL>")?,
        method: Method::GET,
        headers: HeaderMap::new(),
        body: None,
    };

    let response = HttpClient::new(curl, collector)
        .request(request)?
        .perform()
        .await?;

    println!("Response: {:?}", response);
    Ok(())
}
