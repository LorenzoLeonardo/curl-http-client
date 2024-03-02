use std::fs;

use async_curl::actor::CurlActor;
use http::{Method, Request, StatusCode};
use tokio::sync::mpsc::channel;
use url::Url;

use crate::collector::{Collector, FileInfo};
use crate::http_client::{Bps, FileSize, HttpClient};
use crate::test::test_setup::{setup_test_environment, MockResponder, ResponderType};

#[tokio::test]
async fn test_upload() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let to_be_uploaded = tempdir.path().join("file_to_be_uploaded.jpg");
    fs::write(to_be_uploaded.as_path(), include_bytes!("sample.jpg")).unwrap();
    let file_size = fs::metadata(to_be_uploaded.as_path()).unwrap().len() as usize;

    let actor = CurlActor::new();
    let collector = Collector::File(FileInfo::path(to_be_uploaded));
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::PUT)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .upload_file_size(FileSize::from(file_size))
        .unwrap()
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
async fn test_upload_with_speed_control() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let to_be_uploaded = tempdir.path().join("file_to_be_uploaded.jpg");
    fs::write(to_be_uploaded.clone(), include_bytes!("sample.jpg")).unwrap();
    let file_size = fs::metadata(to_be_uploaded.as_path()).unwrap().len() as usize;

    let actor = CurlActor::new();
    let collector = Collector::File(FileInfo::path(to_be_uploaded));
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::PUT)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .upload_file_size(FileSize::from(file_size))
        .unwrap()
        .upload_speed(Bps::from(40000000))
        .unwrap()
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
async fn test_upload_with_transfer_speed_sender() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let to_be_uploaded = tempdir.path().join("file_to_be_uploaded.jpg");
    fs::write(to_be_uploaded.clone(), include_bytes!("sample.jpg")).unwrap();

    let file_size = fs::metadata(to_be_uploaded.as_path()).unwrap().len() as usize;

    let actor = CurlActor::new();

    let (tx, mut rx) = channel(1);

    let file_info = FileInfo::path(to_be_uploaded).with_transfer_speed_sender(tx);
    let collector = Collector::File(file_info);
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::PUT)
        .body(None)
        .unwrap();

    let handle = tokio::spawn(async move {
        while let Some(speed) = rx.recv().await {
            println!("Upload Speed: {} kB/s", speed.as_bytes_per_sec());
        }
    });

    let response = HttpClient::new(collector)
        .upload_file_size(FileSize::from(file_size))
        .unwrap()
        .upload_speed(Bps::from(40000000))
        .unwrap()
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

    handle.abort();
}

#[tokio::test]
async fn test_upload_with_headers() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let to_be_uploaded = tempdir.path().join("file_to_be_uploaded.jpg");
    fs::write(to_be_uploaded.as_path(), include_bytes!("sample.jpg")).unwrap();
    let file_size = fs::metadata(to_be_uploaded.as_path()).unwrap().len() as usize;

    let actor = CurlActor::new();
    let collector = Collector::FileAndHeaders(FileInfo::path(to_be_uploaded), Vec::new());
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::PUT)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .upload_file_size(FileSize::from(file_size))
        .unwrap()
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
async fn test_upload_sync() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let to_be_uploaded = tempdir.path().join("file_to_be_uploaded.jpg");
    fs::write(to_be_uploaded.as_path(), include_bytes!("sample.jpg")).unwrap();
    let file_size = fs::metadata(to_be_uploaded.as_path()).unwrap().len() as usize;

    let collector = Collector::File(FileInfo::path(to_be_uploaded));
    let request = Request::builder()
        .uri(target_url.as_str())
        .method(Method::PUT)
        .body(None)
        .unwrap();
    let response = HttpClient::new(collector)
        .upload_file_size(FileSize::from(file_size))
        .unwrap()
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
