use async_curl::actor::CurlActor;
use http::{HeaderMap, Method};
use url::Url;

use crate::collector::{Collector, ExtendedHandler};
use crate::http_client::HttpClient;
use crate::request::HttpRequest;
use crate::test::test_setup::{setup_test_environment, MockResponder, ResponderType};

#[tokio::test]
async fn test_with_complete_headers() {
    let responder = MockResponder::new(ResponderType::Body("test body".as_bytes().to_vec()));
    let (server, _tempdir) = setup_test_environment(responder).await;
    let target_url = Url::parse(format!("{}/test", server.uri()).as_str()).unwrap();

    let curl = CurlActor::new();
    let collector = Collector::RamAndHeaders(Vec::new(), Vec::new());
    let request = HttpRequest {
        url: target_url,
        method: Method::GET,
        headers: HeaderMap::new(),
        body: None,
    };
    let mut response = HttpClient::new(curl, collector)
        .request(request)
        .unwrap()
        .send_request()
        .await
        .unwrap();

    let (body, headers) = response.get_ref().get_response_body_and_headers();

    println!("body: {:?}", body);
    println!("headers: {:?}", headers);
    println!("status: {:?}", response.response_code().unwrap());

    assert_eq!(body.unwrap(), "test body".as_bytes().to_vec());
    assert_eq!(response.response_code().unwrap(), 200);
}
