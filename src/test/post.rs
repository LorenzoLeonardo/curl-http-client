use async_curl::CurlActor;
use http::{Method, Request, StatusCode};
use url::Url;

use crate::collector::Collector;
use crate::http_client::HttpClient;
use crate::test::test_setup::{setup_test_environment, MockResponder, ResponderType};

#[tokio::test]
async fn test_post() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
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

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*response.body(), None);
    assert!(!response.headers().is_empty());
}

#[tokio::test]
async fn test_post_none() {
    let responder = MockResponder::new(ResponderType::Body(Vec::new()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::POST)
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

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*response.body(), None);
    assert!(!response.headers().is_empty());
}

#[tokio::test]
async fn test_post_none_no_option() {
    let responder = MockResponder::new(ResponderType::Body(Vec::new()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::POST)
        .body(Vec::new())
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .nonblocking(actor)
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*response.body(), None);
    assert!(!response.headers().is_empty());
}

#[tokio::test]
async fn test_post_with_headers() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let actor = CurlActor::new();
    let collector = Collector::RamAndHeaders(Vec::new(), Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
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
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*response.body(), None);
    assert!(!response.headers().is_empty());
}

#[tokio::test]
async fn test_post_sync() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let collector = Collector::Ram(Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::POST)
        .body(Some("test body".as_bytes().to_vec()))
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .blocking()
        .perform()
        .unwrap();

    println!("Response: {:?}", response);
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*response.body(), None);
    assert!(!response.headers().is_empty());
}

#[tokio::test]
async fn test_post_sync_not_option() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let collector = Collector::Ram(Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::POST)
        .body("test body".as_bytes().to_vec())
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .blocking()
        .perform()
        .unwrap();

    println!("Response: {:?}", response);
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*response.body(), None);
    assert!(!response.headers().is_empty());
}

#[tokio::test]
async fn test_post_async_not_option() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::POST)
        .body("test body".as_bytes().to_vec())
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .nonblocking(actor)
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*response.body(), None);
    assert!(!response.headers().is_empty());
}

#[tokio::test]
async fn test_post_sync_not_option_empty_string() {
    let responder = MockResponder::new(ResponderType::Body(Vec::new()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let collector = Collector::Ram(Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::POST)
        .body(Vec::new())
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .blocking()
        .perform()
        .unwrap();

    println!("Response: {:?}", response);
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*response.body(), None);
    assert!(!response.headers().is_empty());
}

#[tokio::test]
async fn test_post_async_not_option_empty_string() {
    let responder = MockResponder::new(ResponderType::Body(Vec::new()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::POST)
        .body(Vec::new())
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .nonblocking(actor)
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*response.body(), None);
    assert!(!response.headers().is_empty());
}
