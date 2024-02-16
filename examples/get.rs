use async_curl::actor::CurlActor;
use curl_http_client::{collector::Collector, http_client::HttpClient, request::HttpRequest};
use http::{HeaderMap, Method};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());

    let request = HttpRequest {
        url: Url::parse("<SOURCE URL>").unwrap(),
        method: Method::GET,
        headers: HeaderMap::new(),
        body: None,
    };

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .nonblocking(actor)
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);
}
