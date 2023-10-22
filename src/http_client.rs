use async_curl::async_curl::AsyncCurl;
use curl::easy::Easy2;
use derive_deref_rs::Deref;
use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, Method, StatusCode};

use crate::{collector::Collector, error::Error, request::HttpRequest, response::HttpResponse};

/// A type-state struct in building the HttpClient.
pub struct Build;
/// A type-state struct in building the HttpClient.
pub struct Perform;

/// The HTTP Client struct that wraps curl Easy2.
pub struct HttpClient<S> {
    /// This is the the actor handler that can be cloned to be able to handle multiple request sender
    /// and a single consumer that is spawned in the background upon creation of this object to be able to achieve
    /// non-blocking I/O during curl perform.
    curl: AsyncCurl<Collector>,
    /// The `Easy2<Collector>` is the Easy2 from curl-rust crate wrapped in this struct to be able to do
    /// asynchronous task during.
    easy: Easy2<Collector>,
    /// This is a type-state builder pattern to help programmers not to mis-used when buding curl settings before perform
    /// operation.
    _state: S,
}

impl HttpClient<Build> {
    /// Creates a new HTTP Client.
    ///
    /// The [`AsyncCurl<Collector>`](https://docs.rs/async-curl/latest/async_curl/async_curl/struct.AsyncCurl.html) is the actor handler that can be cloned to be able to handle multiple request sender
    /// and a single consumer that is spawned in the background upon creation of this object to be able to achieve
    /// non-blocking I/O during curl perform.
    ///
    /// The Collector is the type of container whether via RAM or via File.
    pub fn new(curl: AsyncCurl<Collector>, collector: Collector) -> Self {
        Self {
            curl,
            easy: Easy2::new(collector),
            _state: Build,
        }
    }

    /// Sets the HTTP request.
    ///
    /// The HttpRequest can be customized by the caller byt setting the Url, Method Type,
    /// Headers and the Body.
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
                // TODO: For Future improvements to handle other Methods
                unimplemented!();
            }
        }
        Ok(HttpClient::<Perform> {
            curl: self.curl,
            easy: self.easy,
            _state: Perform,
        })
    }

    /// Set a point to resume transfer from
    ///
    /// Specify the offset in bytes you want the transfer to start from.
    ///
    /// By default this option is 0 and corresponds to
    /// `CURLOPT_RESUME_FROM_LARGE`.
    pub fn resume_from(mut self, offset: BytesOffset) -> Result<Self, Error> {
        self.easy
            .resume_from(*offset as u64)
            .map_err(|e| Error::Curl(e.to_string()))?;
        Ok(self)
    }

    /// Rate limit data download speed
    ///
    /// If a download exceeds this speed (counted in bytes per second) on
    /// cumulative average during the transfer, the transfer will pause to keep
    /// the average rate less than or equal to the parameter value.
    ///
    /// By default this option is not set (unlimited speed) and corresponds to
    /// `CURLOPT_MAX_RECV_SPEED_LARGE`.
    pub fn download_speed(mut self, speed: BytesPerSec) -> Result<Self, Error> {
        self.easy
            .max_recv_speed(*speed)
            .map_err(|e| Error::Curl(e.to_string()))?;
        Ok(self)
    }

    /// Set the size of the input file to send off.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_INFILESIZE_LARGE`.
    pub fn upload_file_size(mut self, size: FileSize) -> Result<Self, Error> {
        self.easy
            .in_filesize(*size as u64)
            .map_err(|e| Error::Curl(e.to_string()))?;
        Ok(self)
    }

    /// Rate limit data upload speed
    ///
    /// If an upload exceeds this speed (counted in bytes per second) on
    /// cumulative average during the transfer, the transfer will pause to keep
    /// the average rate less than or equal to the parameter value.
    ///
    /// By default this option is not set (unlimited speed) and corresponds to
    /// `CURLOPT_MAX_SEND_SPEED_LARGE`.
    pub fn upload_speed(mut self, speed: BytesPerSec) -> Result<Self, Error> {
        self.easy
            .max_send_speed(*speed)
            .map_err(|e| Error::Curl(e.to_string()))?;
        Ok(self)
    }
}

impl HttpClient<Perform> {
    /// This will perform the curl operation asynchronously.
    /// This becomes a non-blocking I/O since the actual perform operation is done
    /// at the actor side.
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

/// A strong type unit when setting download speed and upload speed
/// in bytes per second.
#[derive(Deref)]
pub struct BytesPerSec(u64);

impl From<u64> for BytesPerSec {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// A strong type unit when offsetting especially in resuming download
#[derive(Deref)]
pub struct BytesOffset(usize);

impl From<usize> for BytesOffset {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// A strong type unit when setting a file size.
#[derive(Deref)]
pub struct FileSize(usize);

impl From<usize> for FileSize {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
