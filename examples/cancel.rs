use std::{path::PathBuf, time::Duration};

use async_curl::actor::CurlActor;
use http::{Method, Request};

use curl_http_client::{
    collector::{AbortPerform, Collector, FileInfo},
    http_client::{Bps, HttpClient},
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let actor = CurlActor::new();
    let abort = AbortPerform::new();

    let abort_listener = abort.clone();
    let handle = tokio::spawn(async move {
        let collector = Collector::File(
            FileInfo::path(PathBuf::from("<FILE PATH TO SAVE>"))
                .with_perform_aborter(abort_listener),
        );
        let request = Request::builder()
            .uri("<SOURCE URL>")
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
}
