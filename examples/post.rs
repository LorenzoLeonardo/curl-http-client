use async_curl::async_curl::AsyncCurl;
use curl_http_client::{collector::Collector, http_client::HttpClient, request::HttpRequest};
use http::{HeaderMap, Method};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let curl = AsyncCurl::new();
    let collector = Collector::Ram(Vec::new());

    let request = HttpRequest {
        url: Url::parse("<TARGET URL>")?,
        method: Method::POST,
        headers: HeaderMap::new(),
        body: Some("test body".as_bytes().to_vec()),
    };

    let response = HttpClient::new(curl, collector)
        .request(request)?
        .perform()
        .await?;

    println!("Response: {:?}", response);
    Ok(())
}
