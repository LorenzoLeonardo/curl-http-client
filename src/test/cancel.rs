use std::{fs::File, time::Duration};

use async_curl::actor::CurlActor;
use http::{Method, Request};
use url::Url;

use crate::{
    collector::{AbortPerform, Collector, FileInfo},
    http_client::{Bps, HttpClient},
    test::test_setup::{setup_test_environment, MockResponder, ResponderType},
};

#[tokio::test]
async fn test_download_was_cancelled() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let save_to = tempdir.path().join("downloaded_file.jpg");
    let actor = CurlActor::new();
    let abort = AbortPerform::new();

    let abort_listener = abort.clone();
    let handle = tokio::spawn(async move {
        let collector =
            Collector::File(FileInfo::path(save_to).with_perform_aborter(abort_listener));
        let request = Request::builder()
            .uri(target_url.as_str())
            .method(Method::GET)
            .body(None)
            .unwrap();

        let response = HttpClient::new(collector)
            .progress(true)
            .unwrap()
            .download_speed(Bps::from(5000000))
            .unwrap()
            .request(request)
            .unwrap()
            .nonblocking(actor)
            .perform()
            .await;
        println!("Response: {:?}", response);
    });

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let mut abort = abort.lock().unwrap();
        *abort = true;
    });

    handle.await.unwrap();

    let mock_file = include_bytes!("sample.jpg");
    let save_to = tempdir.path().join("downloaded_file.jpg");

    let downloaded_file = File::open(save_to).unwrap();

    // It must be partially downloaded if cancelled was successful,
    // so determine for the size of the given file and the result.
    assert!(downloaded_file.metadata().unwrap().len() < mock_file.len() as u64);
}

#[tokio::test]
async fn test_download_was_not_cancelled() {
    let responder = MockResponder::new(ResponderType::File);
    let (server, tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let save_to = tempdir.path().join("downloaded_file.jpg");
    let actor = CurlActor::new();
    let abort = AbortPerform::new();

    let abort_listener = abort.clone();
    let handle = tokio::spawn(async move {
        let collector =
            Collector::File(FileInfo::path(save_to).with_perform_aborter(abort_listener));
        let request = Request::builder()
            .uri(target_url.as_str())
            .method(Method::GET)
            .body(None)
            .unwrap();

        let response = HttpClient::new(collector)
            .progress(true)
            .unwrap()
            .download_speed(Bps::from(5000000))
            .unwrap()
            .request(request)
            .unwrap()
            .nonblocking(actor)
            .perform()
            .await;
        println!("Response: {:?}", response);
    });

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let mut abort = abort.lock().unwrap();
        *abort = false;
    });

    handle.await.unwrap();

    let mock_file = include_bytes!("sample.jpg");
    let save_to = tempdir.path().join("downloaded_file.jpg");

    let downloaded_file = File::open(save_to).unwrap();

    // If not cancelled, the file downloaded must be completed.
    assert!(downloaded_file.metadata().unwrap().len() == mock_file.len() as u64);
}
