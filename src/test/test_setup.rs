use tempdir::TempDir;
use wiremock::{matchers::path, Mock, MockServer, Request, Respond, ResponseTemplate};

pub struct MockResponder;

impl Respond for MockResponder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        println!("Request: {:?}", request);
        ResponseTemplate::new(200)
    }
}

pub async fn setup_test_environment() -> (MockServer, TempDir) {
    let mock_server = MockServer::start().await;
    let tempdir = TempDir::new_in("./", "test").unwrap();

    Mock::given(path("/test"))
        .respond_with(MockResponder)
        .mount(&mock_server)
        .await;

    (mock_server, tempdir)
}
