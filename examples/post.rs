use async_curl::CurlActor;
use curl_http_client::{collector::Collector, http_client::HttpClient};
use http::{Method, Request};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());

    let request = Request::builder()
        .uri("<TARGET URL>")
        .method(Method::POST)
        .body(Some("test body".as_bytes().to_vec()))
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .nonblocking(actor)
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);
    Ok(())
}
