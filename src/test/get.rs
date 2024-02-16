use async_curl::actor::CurlActor;
use http::{HeaderMap, Method, StatusCode};
use url::Url;

use crate::collector::Collector;
use crate::http_client::HttpClient;
use crate::request::HttpRequest;
use crate::test::test_setup::{setup_test_environment, MockResponder, ResponderType};

#[tokio::test]
async fn test_get() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());
    let request = HttpRequest {
        url: target_url,
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
    assert_eq!(response.status_code, StatusCode::OK);
    assert_eq!(response.body.unwrap(), "test body".as_bytes().to_vec());
    assert!(!response.headers.is_empty());
}

#[tokio::test]
async fn test_get_with_headers() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let actor = CurlActor::new();
    let collector = Collector::RamAndHeaders(Vec::new(), Vec::new());
    let request = HttpRequest {
        url: target_url,
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
    assert_eq!(response.status_code, StatusCode::OK);
    assert_eq!(response.body.unwrap(), "test body".as_bytes().to_vec());
    assert!(!response.headers.is_empty());
}

#[tokio::test]
async fn test_get_sync() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let collector = Collector::Ram(Vec::new());
    let request = HttpRequest {
        url: target_url,
        method: Method::GET,
        headers: HeaderMap::new(),
        body: None,
    };
    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .blocking()
        .perform()
        .unwrap();

    println!("Response: {:?}", response);
    assert_eq!(response.status_code, StatusCode::OK);
    assert_eq!(response.body.unwrap(), "test body".as_bytes().to_vec());
    assert!(!response.headers.is_empty());
}
