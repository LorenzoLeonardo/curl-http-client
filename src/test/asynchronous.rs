use async_curl::actor::CurlActor;
use futures::future;
use http::{HeaderMap, Method, StatusCode};
use url::Url;

use crate::collector::Collector;
use crate::http_client::HttpClient;
use crate::request::HttpRequest;
use crate::test::test_setup::{setup_test_environment, MockResponder, ResponderType};

#[tokio::test]
async fn test_across_multiple_threads() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let curl = CurlActor::new();
    let collector = Collector::Ram(Vec::new());
    let request = HttpRequest {
        url: target_url,
        method: Method::GET,
        headers: HeaderMap::new(),
        body: None,
    };
    const NUM_CONCURRENT: usize = 100;

    let mut handles = Vec::new();
    for _n in 0..NUM_CONCURRENT {
        let curl = curl.clone();
        let collector = collector.clone();
        let request = request.clone();
        let handle = tokio::spawn(async move {
            let response = HttpClient::new(collector)
                .request(request)
                .unwrap()
                .nonblocking(curl)
                .perform()
                .await
                .unwrap();
            println!("Response: {:?}", response);
            assert_eq!(response.status_code, StatusCode::OK);
            assert_eq!(response.body.unwrap(), "test body".as_bytes().to_vec());
        });
        handles.push(handle);
    }

    let results: Vec<Result<_, _>> = future::join_all(handles).await;

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(res) => {
                println!("Task {} result: {:?}", i + 1, res);
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}
