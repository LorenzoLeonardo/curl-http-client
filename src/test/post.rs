use async_curl::actor::CurlActor;
use http::{HeaderMap, Method, StatusCode};
use url::Url;

use crate::collector::Collector;
use crate::http_client::HttpClient;
use crate::request::HttpRequest;
use crate::test::test_setup::{setup_test_environment, MockResponder, ResponderType};

#[tokio::test]
async fn test_post() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let curl = CurlActor::new();
    let collector = Collector::Ram(Vec::new());
    let request = HttpRequest {
        url: target_url,
        method: Method::POST,
        headers: HeaderMap::new(),
        body: Some("test body".as_bytes().to_vec()),
    };
    let response = HttpClient::new(curl, collector)
        .request(request)
        .unwrap()
        .perform()
        .await
        .unwrap();

    println!("Response: {:?}", response);
    assert_eq!(response.status_code, StatusCode::OK);
    assert_eq!(response.body.unwrap().len(), 0);
}
