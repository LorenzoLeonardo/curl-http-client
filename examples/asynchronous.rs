use async_curl::actor::CurlActor;
use curl_http_client::{collector::Collector, http_client::HttpClient, request::HttpRequest};
use futures::future;
use http::{HeaderMap, Method};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    const NUM_CONCURRENT: usize = 5;

    let curl = CurlActor::new();
    let mut handles = Vec::new();

    for _n in 0..NUM_CONCURRENT {
        let curl = curl.clone();

        let handle = tokio::spawn(async move {
            let collector = Collector::Ram(Vec::new());
            let request = HttpRequest {
                url: Url::parse("https://www.rust-lang.org/").unwrap(),
                method: Method::GET,
                headers: HeaderMap::new(),
                body: None,
            };

            let response = HttpClient::new(curl, collector)
                .request(request)
                .unwrap()
                .perform()
                .await
                .unwrap();
            println!("Response: {:?}", response);
        });
        handles.push(handle);
    }

    let results: Vec<Result<_, _>> = future::join_all(handles).await;

    for (_i, result) in results.into_iter().enumerate() {
        result.unwrap();
    }
}
