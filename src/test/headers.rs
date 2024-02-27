use std::fs;

use async_curl::actor::CurlActor;
use http::{Method, Request};
use url::Url;

use crate::collector::{Collector, ExtendedHandler, FileInfo};
use crate::http_client::HttpClient;
use crate::test::test_setup::{setup_test_environment, MockResponder, ResponderType};

#[tokio::test]
async fn test_with_complete_headers_ram_and_header() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let actor = CurlActor::new();
    let collector = Collector::RamAndHeaders(Vec::new(), Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::GET)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .nonblocking(actor)
        .send_request()
        .await
        .unwrap();

    let (body, headers) = response.get_ref().get_response_body_and_headers();

    println!("body: {:?}", body);
    println!("headers: {:?}", headers);
    println!("status: {:?}", response.response_code().unwrap());

    assert!(headers.is_some());
    assert_eq!(body.unwrap(), "test body".as_bytes().to_vec());
    assert_eq!(response.response_code().unwrap(), 200);
}

#[tokio::test]
async fn test_with_complete_headers_file_and_headers() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let save_to = tempdir.path().join("downloaded_file.jpg");
    let actor = CurlActor::new();
    let collector = Collector::FileAndHeaders(FileInfo::path(save_to.clone()), Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::GET)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .nonblocking(actor)
        .send_request()
        .await
        .unwrap();

    let (body, headers) = response.get_ref().get_response_body_and_headers();

    println!("body: {:?}", body);
    println!("headers: {:?}", headers);
    println!("status: {:?}", response.response_code().unwrap());

    assert!(headers.is_some());
    assert_eq!(body, None);
    assert_eq!(response.response_code().unwrap(), 200);
    assert_eq!(fs::read(save_to).unwrap(), include_bytes!("sample.jpg"));
}

#[tokio::test]
async fn test_with_complete_headers_ram() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::GET)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .nonblocking(actor)
        .send_request()
        .await
        .unwrap();

    let (body, headers) = response.get_ref().get_response_body_and_headers();

    println!("body: {:?}", body);
    println!("headers: {:?}", headers);
    println!("status: {:?}", response.response_code().unwrap());

    assert!(headers.is_none());
    assert_eq!(body.unwrap(), "test body".as_bytes().to_vec());
    assert_eq!(response.response_code().unwrap(), 200);
}

#[tokio::test]
async fn test_with_complete_headers_file() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let save_to = tempdir.path().join("downloaded_file.jpg");
    let actor = CurlActor::new();
    let collector = Collector::File(FileInfo::path(save_to.clone()));
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::GET)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .nonblocking(actor)
        .send_request()
        .await
        .unwrap();

    let (body, headers) = response.get_ref().get_response_body_and_headers();

    println!("body: {:?}", body);
    println!("headers: {:?}", headers);
    println!("status: {:?}", response.response_code().unwrap());

    assert!(headers.is_none());
    assert_eq!(body, None);
    assert_eq!(response.response_code().unwrap(), 200);
    assert_eq!(fs::read(save_to).unwrap(), include_bytes!("sample.jpg"));
}

#[tokio::test]
async fn test_with_complete_headers_ram_and_header_sync() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let collector = Collector::RamAndHeaders(Vec::new(), Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::GET)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request)
        .unwrap()
        .blocking()
        .send_request()
        .unwrap();

    let (body, headers) = response.get_ref().get_response_body_and_headers();

    println!("body: {:?}", body);
    println!("headers: {:?}", headers);
    println!("status: {:?}", response.response_code().unwrap());

    assert!(headers.is_some());
    assert_eq!(body.unwrap(), "test body".as_bytes().to_vec());
    assert_eq!(response.response_code().unwrap(), 200);
}
