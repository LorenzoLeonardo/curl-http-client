use async_curl::actor::CurlActor;
use curl_http_client::{collector::Collector, http_client::HttpClient, request::HttpRequest};
use http::{HeaderMap, Method};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let curl = CurlActor::new();
    let collector = Collector::Ram(Vec::new());

    let request = HttpRequest {
        url: Url::parse("<TARGET URL>").unwrap(),
        method: Method::POST,
        headers: HeaderMap::new(),
        body: Some("test body".as_bytes().to_vec()),
    };

    let response = HttpClient::new(curl, collector)
        .request(request)
        .unwrap()
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);
    Ok(())
}
