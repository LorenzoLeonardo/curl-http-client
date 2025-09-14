use std::sync::Arc;

use async_curl::CurlActor;
use http::{Method, Request, StatusCode};
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;
use url::Url;

use crate::collector::Collector;
use crate::http_client::HttpClient;
use crate::test::test_setup::{setup_test_environment, MockResponder, ResponderType};
use crate::StreamHandler;

#[tokio::test]
async fn test_streaming() {
    let responder = MockResponder::new(ResponderType::Stream);
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let (tx, mut rx) = unbounded_channel();
    let actor = CurlActor::new();
    let stream = StreamHandler {
        chunk_sender: tx,
        abort: None,
    };

    let collector = Collector::Streaming(stream, Vec::new());
    let result = Arc::new(Mutex::new(Vec::new()));
    let inner = result.clone();
    let handle = tokio::spawn(async move {
        while let Some(chunk) = rx.recv().await {
            println!("Recieving Data: {}", chunk.len());
            inner.lock().await.extend_from_slice(&chunk);
        }
        println!("Streaming done");
    });

    let request = Request::builder()
        .uri(target_url.as_str())
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
    println!("Headers: {:?}", response.headers());

    handle.await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*response.body(), None);
    let result = result.lock().await.clone();
    let expected = include_bytes!("sample.jpg").to_vec();

    println!(
        "Size Result: {} Size Expected: {}",
        result.len(),
        expected.len()
    );

    assert_eq!(result, expected);
}
