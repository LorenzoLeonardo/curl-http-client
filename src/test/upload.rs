use std::fs;

use async_curl::actor::CurlActor;
use http::{HeaderMap, Method, StatusCode};
use tokio::sync::mpsc::channel;
use url::Url;

use crate::collector::{Collector, FileInfo};
use crate::http_client::{BytesPerSec, FileSize, HttpClient};
use crate::request::HttpRequest;
use crate::test::test_setup::{setup_test_environment, MockResponder, ResponderType};

#[tokio::test]
async fn test_upload() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let to_be_uploaded = tempdir.path().join("file_to_be_uploaded.jpg");
    fs::write(to_be_uploaded.as_path(), include_bytes!("sample.jpg")).unwrap();
    let file_size = fs::metadata(to_be_uploaded.as_path()).unwrap().len() as usize;

    let curl = CurlActor::new();
    let collector = Collector::File(FileInfo::path(to_be_uploaded));
    let request = HttpRequest {
        url: target_url,
        method: Method::PUT,
        headers: HeaderMap::new(),
        body: None,
    };
    let response = HttpClient::new(curl, collector)
        .upload_file_size(FileSize::from(file_size))
        .unwrap()
        .request(request)
        .unwrap()
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);
    assert_eq!(response.status_code, StatusCode::OK);
    assert_eq!(response.body, None);
}

#[tokio::test]
async fn test_upload_with_speed_control() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let to_be_uploaded = tempdir.path().join("file_to_be_uploaded.jpg");
    fs::write(to_be_uploaded.clone(), include_bytes!("sample.jpg")).unwrap();
    let file_size = fs::metadata(to_be_uploaded.as_path()).unwrap().len() as usize;

    let curl = CurlActor::new();
    let collector = Collector::File(FileInfo::path(to_be_uploaded));
    let request = HttpRequest {
        url: target_url,
        method: Method::PUT,
        headers: HeaderMap::new(),
        body: None,
    };
    let response = HttpClient::new(curl, collector)
        .upload_file_size(FileSize::from(file_size))
        .unwrap()
        .upload_speed(BytesPerSec::from(40000000))
        .unwrap()
        .request(request)
        .unwrap()
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);
    assert_eq!(response.status_code, StatusCode::OK);
    assert_eq!(response.body, None);
}

#[tokio::test]
async fn test_upload_with_transfer_speed_sender() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let to_be_uploaded = tempdir.path().join("file_to_be_uploaded.jpg");
    fs::write(to_be_uploaded.clone(), include_bytes!("sample.jpg")).unwrap();

    let file_size = fs::metadata(to_be_uploaded.as_path()).unwrap().len() as usize;

    let curl = CurlActor::new();

    let (tx, mut rx) = channel(1);

    let file_info = FileInfo::path(to_be_uploaded).with_transfer_speed_sender(tx);
    let collector = Collector::File(file_info);
    let request = HttpRequest {
        url: target_url,
        method: Method::PUT,
        headers: HeaderMap::new(),
        body: None,
    };

    let handle = tokio::spawn(async move {
        while let Some(speed) = rx.recv().await {
            println!("Upload Speed: {} kB/s", speed.as_bytes_per_sec());
        }
    });

    let response = HttpClient::new(curl, collector)
        .upload_file_size(FileSize::from(file_size))
        .unwrap()
        .upload_speed(BytesPerSec::from(40000000))
        .unwrap()
        .request(request)
        .unwrap()
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);
    assert_eq!(response.status_code, StatusCode::OK);
    assert_eq!(response.body, None);

    handle.abort();
}
