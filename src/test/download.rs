use std::fs;

use async_curl::async_curl::AsyncCurl;
use http::{HeaderMap, Method, StatusCode};
use test_case::test_case;
use url::Url;

use crate::collector::{Collector, FileInfo};
use crate::http_client::{BytesOffset, BytesPerSec, HttpClient};
use crate::request::HttpRequest;
use crate::test::test_setup::{setup_test_environment, MockResponder, ResponderType};

#[tokio::test]
async fn test_download() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let save_to = tempdir.path().join("downloaded_file.jpg");
    let curl = AsyncCurl::new();
    let collector = Collector::File(FileInfo::path(save_to.clone()));
    let request = HttpRequest {
        url: target_url,
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
    assert_eq!(response.status_code, StatusCode::OK);
    assert_eq!(response.body, None);
    assert_eq!(fs::read(save_to).unwrap(), include_bytes!("sample.jpg"));
}

#[tokio::test]
async fn test_download_with_speed_control() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let save_to = tempdir.path().join("downloaded_file.jpg");
    let curl = AsyncCurl::new();
    let collector = Collector::File(FileInfo::path(save_to.clone()));
    let request = HttpRequest {
        url: target_url,
        method: Method::GET,
        headers: HeaderMap::new(),
        body: None,
    };
    let response = HttpClient::new(curl, collector)
        .download_speed(BytesPerSec::from(4000000))
        .unwrap()
        .request(request)
        .unwrap()
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);
    assert_eq!(response.status_code, StatusCode::OK);
    assert_eq!(response.body, None);
    assert_eq!(fs::read(save_to).unwrap(), include_bytes!("sample.jpg"));
}

#[test_case(4500, StatusCode::PARTIAL_CONTENT; "Offset 4500 bytes")]
#[test_case(0, StatusCode::OK ; "Offset 0 bytes")]
#[test_case(include_bytes!("sample.jpg").len(), StatusCode::PARTIAL_CONTENT ; "Offset max bytes")]
#[tokio::test]
async fn test_resume_download(offset: usize, expected_status_code: StatusCode) {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let save_to = tempdir.path().join("downloaded_file.jpg");

    let partial_saved_file = include_bytes!("sample.jpg");
    fs::write(save_to.as_path(), &partial_saved_file[0..offset]).unwrap();

    let partial_file_size = fs::metadata(save_to.as_path()).unwrap().len() as usize;

    let curl = AsyncCurl::new();
    let collector = Collector::File(FileInfo::path(save_to.clone()));
    let request = HttpRequest {
        url: target_url,
        method: Method::GET,
        headers: HeaderMap::new(),
        body: None,
    };
    let response = HttpClient::new(curl, collector)
        .resume_from(BytesOffset::from(partial_file_size))
        .unwrap()
        .request(request)
        .unwrap()
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);
    assert_eq!(response.status_code, expected_status_code);
    assert_eq!(response.body, None);
    assert_eq!(fs::read(save_to).unwrap(), include_bytes!("sample.jpg"));
}
