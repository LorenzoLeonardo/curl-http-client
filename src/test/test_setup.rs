use std::str::FromStr;

use http::StatusCode;
use tempfile::TempDir;
use wiremock::{
    http::{HeaderName, HeaderValue, HeaderValues, Method},
    matchers::path,
    Mock, MockServer, Request, Respond, ResponseTemplate,
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
        match request.method {
            Method::Get => match &self.responder {
                ResponderType::File => {
                    let mock_file = include_bytes!("sample.jpg");
                    let header_name = HeaderName::from_str("range").unwrap();
                    let total_file_size = mock_file.len();
                    println!("Request: {:?}", request);
                    if let Some(value) = request.headers.get(&header_name) {
                        let offset = parse_range(value).unwrap() as usize;
                        println!("Offset: {}", offset);

                        let content_length = format!("{}", total_file_size - offset);
                        println!("Content-Length: {}", content_length);
                        let content_range = format!(
                            "bytes {}-{}/{}",
                            offset,
                            total_file_size - 1,
                            total_file_size
                        );
                        println!("Content-Range: {}", content_range);

                        ResponseTemplate::new(StatusCode::PARTIAL_CONTENT)
                            .append_header(
                                HeaderName::from_str("Content-Range").unwrap(),
                                HeaderValue::from_str(content_range.as_str()).unwrap(),
                            )
                            .append_header(
                                HeaderName::from_str("Content-Length").unwrap(),
                                HeaderValue::from_str(content_length.as_str()).unwrap(),
                            )
                            .append_header(
                                HeaderName::from_str("Accept-Ranges").unwrap(),
                                HeaderValue::from_str("bytes").unwrap(),
                            )
                            .set_body_bytes(&mock_file[offset..])
                    } else {
                        let contents = include_bytes!("sample.jpg");
                        ResponseTemplate::new(StatusCode::OK).set_body_bytes(contents.as_slice())
                    }
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
                    let header_name = HeaderName::from_str("content-range").unwrap();
                    let content = include_bytes!("sample.jpg").to_vec();
                    println!("Request Header: {:?}", request.headers);
                    if let Some(value) = request.headers.get(&header_name) {
                        let last_byte = parse_content_range(value).unwrap() as usize;
                        let mut partial_content = content[0..last_byte].to_vec();

                        partial_content.extend_from_slice(request.body.as_slice());
                        assert_eq!(partial_content, content);
                        let body = r#"{"message":"Partial upload successful","partial_upload_id":"ABC123"}"#;
                        ResponseTemplate::new(StatusCode::PARTIAL_CONTENT)
                            .append_header(
                                HeaderName::from_str("Content-Range").unwrap(),
                                value[0].clone(),
                            )
                            .append_header("Content-Type", "application/json")
                            .set_body_bytes(body.as_bytes())
                    } else {
                        assert_eq!(content, request.body);
                        ResponseTemplate::new(StatusCode::OK)
                    }
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

fn parse_range(input: &HeaderValues) -> Option<u64> {
    let input = input.to_string();
    if let Some(start_pos) = input.find('=') {
        if let Some(end_pos) = input.rfind('-') {
            let numeric_value = &input[start_pos + 1..end_pos];
            numeric_value.parse::<u64>().ok()
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_content_range(input: &HeaderValues) -> Option<u64> {
    let input = input.to_string();
    println!("HeaderValues:{}", input);
    let parts: Vec<&str> = input.split('-').collect();
    if parts.len() == 2 {
        let start_byte = parts[0].replace("[\"bytes ", "");
        start_byte.parse::<u64>().ok()
    } else {
        None
    }
}

pub async fn setup_test_environment(responder: MockResponder) -> (MockServer, TempDir) {
    let mock_server = MockServer::start().await;
    let tempdir = TempDir::with_prefix_in("test", "./").unwrap();

    Mock::given(path("/test"))
        .respond_with(responder)
        .mount(&mock_server)
        .await;

    (mock_server, tempdir)
}
