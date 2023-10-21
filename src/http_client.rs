use async_curl::async_curl::AsyncCurl;
use curl::easy::Easy2;
use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, Method, StatusCode};

use crate::{collector::Collector, error::Error, request::HttpRequest, response::HttpResponse};

pub struct Build;
pub struct Perform;

pub struct HttpClient<S> {
    curl: AsyncCurl<Collector>,
    easy: Easy2<Collector>,
    _state: S,
}

impl HttpClient<Build> {
    pub fn new(curl: AsyncCurl<Collector>, collector: Collector) -> Self {
        Self {
            curl,
            easy: Easy2::new(collector),
            _state: Build,
        }
    }

    pub fn request(mut self, request: HttpRequest) -> Result<HttpClient<Perform>, Error> {
        self.easy.url(&request.url.to_string()[..]).map_err(|e| {
            eprintln!("{:?}", e);
            Error::Curl(e.to_string())
        })?;

        let mut headers = curl::easy::List::new();
        request.headers.iter().try_for_each(|(name, value)| {
            headers
                .append(&format!(
                    "{}: {}",
                    name,
                    value.to_str().map_err(|_| Error::Other(format!(
                        "invalid {} header value {:?}",
                        name,
                        value.as_bytes()
                    )))?
                ))
                .map_err(|e| {
                    eprintln!("{:?}", e);
                    Error::Curl(e.to_string())
                })
        })?;

        self.easy.http_headers(headers).map_err(|e| {
            eprintln!("{:?}", e);
            Error::Curl(e.to_string())
        })?;

        match request.method {
            Method::POST => {
                self.easy
                    .post(true)
                    .map_err(|e| Error::Curl(e.to_string()))?;
                if let Some(body) = request.body {
                    self.easy.post_field_size(body.len() as u64).map_err(|e| {
                        eprintln!("{:?}", e);
                        Error::Curl(e.to_string())
                    })?;
                    self.easy.post_fields_copy(body.as_slice()).map_err(|e| {
                        eprintln!("{:?}", e);
                        Error::Curl(e.to_string())
                    })?;
                }
            }
            Method::GET => {
                self.easy
                    .get(true)
                    .map_err(|e| Error::Curl(e.to_string()))?;
            }
            Method::PUT => {
                self.easy
                    .upload(true)
                    .map_err(|e| Error::Curl(e.to_string()))?;
            }
            _ => {
                unimplemented!();
            }
        }
        Ok(HttpClient::<Perform> {
            curl: self.curl,
            easy: self.easy,
            _state: Perform,
        })
    }
}

impl HttpClient<Perform> {
    pub async fn perform(self) -> Result<HttpResponse, Error> {
        let mut easy = self.curl.send_request(self.easy).await.map_err(|e| {
            eprintln!("{:?}", e);
            Error::Curl(e.to_string())
        })?;

        let data = easy.get_ref().get_response_body().take();
        let status_code = easy.response_code().map_err(|e| {
            eprintln!("{:?}", e);
            Error::Curl(e.to_string())
        })? as u16;
        let response_header = easy
            .content_type()
            .map_err(|e| {
                eprintln!("{:?}", e);
                Error::Curl(e.to_string())
            })?
            .map(|content_type| {
                Ok(vec![(
                    CONTENT_TYPE,
                    HeaderValue::from_str(content_type).map_err(|err| {
                        eprintln!("{:?}", err);
                        Error::Curl(err.to_string())
                    })?,
                )]
                .into_iter()
                .collect::<HeaderMap>())
            })
            .transpose()?
            .unwrap_or_else(HeaderMap::new);

        Ok(HttpResponse {
            status_code: StatusCode::from_u16(status_code).map_err(|err| {
                eprintln!("{:?}", err);
                Error::Curl(err.to_string())
            })?,
            headers: response_header,
            body: data,
        })
    }
}
