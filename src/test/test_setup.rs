use http::StatusCode;
use tempdir::TempDir;
use wiremock::{
    http::Method, matchers::path, Mock, MockServer, Request, Respond, ResponseTemplate,
};

pub enum ResponderType {
    File,
    Body(Vec<u8>),
}
pub struct MockResponder {
    responder: ResponderType,
}

impl MockResponder {
    pub fn new(responder: ResponderType) -> Self {
        Self { responder }
    }
}

impl Respond for MockResponder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        println!("Request: {:?}", request);
        match request.method {
            Method::Get => match &self.responder {
                ResponderType::File => {
                    let contents = include_bytes!("sample.jpg");
                    ResponseTemplate::new(StatusCode::OK).set_body_bytes(contents.as_slice())
                }
                ResponderType::Body(body) => {
                    ResponseTemplate::new(StatusCode::OK).set_body_bytes(body.as_slice())
                }
            },
            Method::Post => match &self.responder {
                ResponderType::File => ResponseTemplate::new(StatusCode::OK),
                ResponderType::Body(body) => {
                    assert_eq!(*body, request.body);
                    ResponseTemplate::new(StatusCode::OK)
                }
            },
            Method::Put => match &self.responder {
                ResponderType::File => {
                    assert_eq!(include_bytes!("sample.jpg").to_vec(), request.body);
                    ResponseTemplate::new(StatusCode::OK)
                }
                ResponderType::Body(body) => {
                    assert_eq!(*body, request.body);
                    ResponseTemplate::new(StatusCode::OK)
                }
            },
            _ => {
                unimplemented!()
            }
        }
    }
}

pub async fn setup_test_environment(responder: MockResponder) -> (MockServer, TempDir) {
    let mock_server = MockServer::start().await;
    let tempdir = TempDir::new_in("./", "test").unwrap();

    Mock::given(path("/test"))
        .respond_with(responder)
        .mount(&mock_server)
        .await;

    (mock_server, tempdir)
}
