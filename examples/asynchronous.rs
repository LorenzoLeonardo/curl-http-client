use async_curl::CurlActor;
use curl_http_client::{collector::Collector, http_client::HttpClient};
use futures::future;
use http::{Method, Request};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    const NUM_CONCURRENT: usize = 5;

    let curl = CurlActor::new();
    let mut handles = Vec::new();

    for _n in 0..NUM_CONCURRENT {
        let actor = curl.clone();

        let handle = tokio::spawn(async move {
            let collector = Collector::Ram(Vec::new());
            let request = Request::builder()
                .uri("https://www.rust-lang.org/")
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
        });
        handles.push(handle);
    }

    let results: Vec<Result<_, _>> = future::join_all(handles).await;

    for (_i, result) in results.into_iter().enumerate() {
        result.unwrap();
    }
}
