use std::{fmt::Debug, path::Path, time::Duration};

use async_curl::actor::Actor;
use curl::easy::{Auth, Easy2, Handler, HttpVersion, ProxyType, SslVersion, TimeCondition};
use derive_deref_rs::Deref;
use http::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    HeaderMap, HeaderValue, Method, Request, Response,
};
use log::trace;

use crate::{collector::ExtendedHandler, error::Error};

/// The HttpClient struct's job is to wrap and build curl Easy2.
pub struct HttpClient<C>
where
    C: Handler + Debug + Send + 'static,
{
    easy: Easy2<C>,
}

impl<C> HttpClient<C>
where
    C: ExtendedHandler + Debug + Send + 'static,
{
    /// Creates a new HTTP Client.
    ///
    /// The C is a generic type to be able to implement a custom HTTP response collector whoever uses this crate.
    /// There is a built-in [`Collector`](https://docs.rs/curl-http-client/latest/curl_http_client/collector/enum.Collector.html) in this crate that can be used store HTTP response body into memory or in a File.
    pub fn new(collector: C) -> Self {
        Self {
            easy: Easy2::new(collector),
        }
    }

    /// This marks the end of the curl builder to be able to do asynchronous operation during perform.
    ///
    /// The parameter trait [`Actor<C>`](https://docs.rs/async-curl/latest/async_curl/actor/trait.Actor.html) is any custom Actor implemented by the user that
    /// must implement a send_request that is non-blocking.
    ///
    /// There is a built-in [`CurlActor`](https://docs.rs/async-curl/latest/async_curl/actor/struct.CurlActor.html) that implements the
    /// [`Actor<C>`](https://docs.rs/async-curl/latest/async_curl/actor/trait.Actor.html) trait that can be cloned
    /// to be able to handle multiple request sender and a single consumer that is spawned in the background to be able to achieve
    /// non-blocking I/O during curl perform.
    pub fn nonblocking<A: Actor<C>>(self, actor: A) -> AsyncPerform<C, A> {
        AsyncPerform::<C, A> {
            actor,
            easy: self.easy,
        }
    }

    /// This marks the end of the curl builder to be able to do synchronous operation during perform.
    pub fn blocking(self) -> SyncPerform<C> {
        SyncPerform::<C> { easy: self.easy }
    }

    /// Sets the HTTP request.
    ///
    /// The HttpRequest can be customized by the caller by setting the Url, Method Type,
    /// Headers and the Body.
    pub fn request<B: CurlBodyRequest>(mut self, request: Request<B>) -> Result<Self, Error<C>> {
        self.easy
            .url(request.uri().to_string().as_str())
            .map_err(|e| {
                trace!("{:?}", e);
                Error::Curl(e)
            })?;

        let mut headers = curl::easy::List::new();

        request.headers().iter().try_for_each(|(name, value)| {
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
                    trace!("{:?}", e);
                    Error::Curl(e)
                })
        })?;

        self.easy.http_headers(headers).map_err(|e| {
            trace!("{:?}", e);
            Error::Curl(e)
        })?;

        match *request.method() {
            Method::POST => {
                self.easy.post(true).map_err(Error::Curl)?;

                if let Some(body) = request.body().get_bytes() {
                    self.easy.post_field_size(body.len() as u64).map_err(|e| {
                        trace!("{:?}", e);
                        Error::Curl(e)
                    })?;
                    self.easy.post_fields_copy(body).map_err(|e| {
                        trace!("{:?}", e);
                        Error::Curl(e)
                    })?;
                }
            }
            Method::GET => {
                self.easy.get(true).map_err(Error::Curl)?;
            }
            Method::PUT => {
                self.easy.upload(true).map_err(Error::Curl)?;
            }
            _ => {
                // TODO: For Future improvements to handle other Methods
                unimplemented!();
            }
        }
        Ok(self)
    }

    /// Set a point to resume transfer from
    ///
    /// Specify the offset in bytes you want the transfer to start from.
    ///
    /// By default this option is 0 and corresponds to
    /// `CURLOPT_RESUME_FROM_LARGE`.
    pub fn resume_from(mut self, offset: BytesOffset) -> Result<Self, Error<C>> {
        self.easy.resume_from(*offset as u64).map_err(Error::Curl)?;
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
    pub fn download_speed(mut self, speed: Bps) -> Result<Self, Error<C>> {
        self.easy.max_recv_speed(*speed).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set the size of the input file to send off.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_INFILESIZE_LARGE`.
    pub fn upload_file_size(mut self, size: FileSize) -> Result<Self, Error<C>> {
        self.easy.in_filesize(*size as u64).map_err(Error::Curl)?;
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
    pub fn upload_speed(mut self, speed: Bps) -> Result<Self, Error<C>> {
        self.easy.max_send_speed(*speed).map_err(Error::Curl)?;
        Ok(self)
    }

    // =========================================================================
    // Names and passwords

    /// Configures the username to pass as authentication for this connection.
    ///
    /// By default this value is not set and corresponds to `CURLOPT_USERNAME`.
    pub fn username(mut self, user: &str) -> Result<Self, Error<C>> {
        self.easy.username(user).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Configures the password to pass as authentication for this connection.
    ///
    /// By default this value is not set and corresponds to `CURLOPT_PASSWORD`.
    pub fn password(mut self, pass: &str) -> Result<Self, Error<C>> {
        self.easy.password(pass).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set HTTP server authentication methods to try
    ///
    /// If more than one method is set, libcurl will first query the site to see
    /// which authentication methods it supports and then pick the best one you
    /// allow it to use. For some methods, this will induce an extra network
    /// round-trip. Set the actual name and password with the `password` and
    /// `username` methods.
    ///
    /// For authentication with a proxy, see `proxy_auth`.
    ///
    /// By default this value is basic and corresponds to `CURLOPT_HTTPAUTH`.
    pub fn http_auth(mut self, auth: &Auth) -> Result<Self, Error<C>> {
        self.easy.http_auth(auth).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Configures the port number to connect to, instead of the one specified
    /// in the URL or the default of the protocol.
    pub fn port(mut self, port: u16) -> Result<Self, Error<C>> {
        self.easy.port(port).map_err(Error::Curl)?;
        Ok(self)
    }

    // /// Verify the certificate's status.
    // ///
    // /// This option determines whether libcurl verifies the status of the server
    // /// cert using the "Certificate Status Request" TLS extension (aka. OCSP
    // /// stapling).
    // ///
    // /// By default this option is set to `false` and corresponds to
    // /// `CURLOPT_SSL_VERIFYSTATUS`.
    // pub fn ssl_verify_status(&mut self, verify: bool) -> Result<(), Error<C>> {
    //     self.setopt_long(curl_sys::CURLOPT_SSL_VERIFYSTATUS, verify as c_long)
    // }

    /// Specify the path to Certificate Authority (CA) bundle
    ///
    /// The file referenced should hold one or more certificates to verify the
    /// peer with.
    ///
    /// This option is by default set to the system path where libcurl's cacert
    /// bundle is assumed to be stored, as established at build time.
    ///
    /// If curl is built against the NSS SSL library, the NSS PEM PKCS#11 module
    /// (libnsspem.so) needs to be available for this option to work properly.
    ///
    /// By default this option is the system defaults, and corresponds to
    /// `CURLOPT_CAINFO`.
    pub fn cainfo<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error<C>> {
        self.easy.cainfo(path).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Specify directory holding CA certificates
    ///
    /// Names a directory holding multiple CA certificates to verify the peer
    /// with. If libcurl is built against OpenSSL, the certificate directory
    /// must be prepared using the openssl c_rehash utility. This makes sense
    /// only when used in combination with the `ssl_verify_peer` option.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_CAPATH`.
    pub fn capath<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error<C>> {
        self.easy.capath(path).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Configures the proxy username to pass as authentication for this
    /// connection.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXYUSERNAME`.
    pub fn proxy_username(mut self, user: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_username(user).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Configures the proxy password to pass as authentication for this
    /// connection.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXYPASSWORD`.
    pub fn proxy_password(mut self, pass: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_password(pass).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set HTTP proxy authentication methods to try
    ///
    /// If more than one method is set, libcurl will first query the site to see
    /// which authentication methods it supports and then pick the best one you
    /// allow it to use. For some methods, this will induce an extra network
    /// round-trip. Set the actual name and password with the `proxy_password`
    /// and `proxy_username` methods.
    ///
    /// By default this value is basic and corresponds to `CURLOPT_PROXYAUTH`.
    pub fn proxy_auth(mut self, auth: &Auth) -> Result<Self, Error<C>> {
        self.easy.proxy_auth(auth).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Provide the URL of a proxy to use.
    ///
    /// By default this option is not set and corresponds to `CURLOPT_PROXY`.
    pub fn proxy(mut self, url: &str) -> Result<Self, Error<C>> {
        self.easy.proxy(url).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Provide port number the proxy is listening on.
    ///
    /// By default this option is not set (the default port for the proxy
    /// protocol is used) and corresponds to `CURLOPT_PROXYPORT`.
    pub fn proxy_port(mut self, port: u16) -> Result<Self, Error<C>> {
        self.easy.proxy_port(port).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set CA certificate to verify peer against for proxy.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXY_CAINFO`.
    pub fn proxy_cainfo(mut self, cainfo: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_cainfo(cainfo).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Specify a directory holding CA certificates for proxy.
    ///
    /// The specified directory should hold multiple CA certificates to verify
    /// the HTTPS proxy with. If libcurl is built against OpenSSL, the
    /// certificate directory must be prepared using the OpenSSL `c_rehash`
    /// utility.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXY_CAPATH`.
    pub fn proxy_capath<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error<C>> {
        self.easy.proxy_capath(path).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set client certificate for proxy.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXY_SSLCERT`.
    pub fn proxy_sslcert(mut self, sslcert: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_sslcert(sslcert).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Specify type of the client SSL certificate for HTTPS proxy.
    ///
    /// The string should be the format of your certificate. Supported formats
    /// are "PEM" and "DER", except with Secure Transport. OpenSSL (versions
    /// 0.9.3 and later) and Secure Transport (on iOS 5 or later, or OS X 10.7
    /// or later) also support "P12" for PKCS#12-encoded files.
    ///
    /// By default this option is "PEM" and corresponds to
    /// `CURLOPT_PROXY_SSLCERTTYPE`.
    pub fn proxy_sslcert_type(mut self, kind: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_sslcert_type(kind).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set the client certificate for the proxy using an in-memory blob.
    ///
    /// The specified byte buffer should contain the binary content of the
    /// certificate, which will be copied into the handle.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_SSLCERT_BLOB`.
    pub fn proxy_sslcert_blob(mut self, blob: &[u8]) -> Result<Self, Error<C>> {
        self.easy.proxy_sslcert_blob(blob).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set private key for HTTPS proxy.
    ///
    /// By default this value is not set and corresponds to
    /// `CURLOPT_PROXY_SSLKEY`.
    pub fn proxy_sslkey(mut self, sslkey: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_sslkey(sslkey).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set type of the private key file for HTTPS proxy.
    ///
    /// The string should be the format of your private key. Supported formats
    /// are "PEM", "DER" and "ENG".
    ///
    /// The format "ENG" enables you to load the private key from a crypto
    /// engine. In this case `ssl_key` is used as an identifier passed to
    /// the engine. You have to set the crypto engine with `ssl_engine`.
    /// "DER" format key file currently does not work because of a bug in
    /// OpenSSL.
    ///
    /// By default this option is "PEM" and corresponds to
    /// `CURLOPT_PROXY_SSLKEYTYPE`.
    pub fn proxy_sslkey_type(mut self, kind: &str) -> Result<Self, Error<C>> {
        self.easy.proxy_sslkey_type(kind).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set the private key for the proxy using an in-memory blob.
    ///
    /// The specified byte buffer should contain the binary content of the
    /// private key, which will be copied into the handle.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_SSLKEY_BLOB`.
    pub fn proxy_sslkey_blob(mut self, blob: &[u8]) -> Result<Self, Error<C>> {
        self.easy.proxy_sslkey_blob(blob).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set passphrase to private key for HTTPS proxy.
    ///
    /// This will be used as the password required to use the `ssl_key`.
    /// You never needed a pass phrase to load a certificate but you need one to
    /// load your private key.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_PROXY_KEYPASSWD`.
    pub fn proxy_key_password(mut self, password: &str) -> Result<Self, Error<C>> {
        self.easy
            .proxy_key_password(password)
            .map_err(Error::Curl)?;
        Ok(self)
    }

    /// Indicates the type of proxy being used.
    ///
    /// By default this option is `ProxyType::Http` and corresponds to
    /// `CURLOPT_PROXYTYPE`.
    pub fn proxy_type(mut self, kind: ProxyType) -> Result<Self, Error<C>> {
        self.easy.proxy_type(kind).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Provide a list of hosts that should not be proxied to.
    ///
    /// This string is a comma-separated list of hosts which should not use the
    /// proxy specified for connections. A single `*` character is also accepted
    /// as a wildcard for all hosts.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_NOPROXY`.
    pub fn noproxy(mut self, skip: &str) -> Result<Self, Error<C>> {
        self.easy.noproxy(skip).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Inform curl whether it should tunnel all operations through the proxy.
    ///
    /// This essentially means that a `CONNECT` is sent to the proxy for all
    /// outbound requests.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_HTTPPROXYTUNNEL`.
    pub fn http_proxy_tunnel(mut self, tunnel: bool) -> Result<Self, Error<C>> {
        self.easy.http_proxy_tunnel(tunnel).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Follow HTTP 3xx redirects.
    ///
    /// Indicates whether any `Location` headers in the response should get
    /// followed.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_FOLLOWLOCATION`.
    pub fn follow_location(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.follow_location(enable).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Force a new connection to be used.
    ///
    /// Makes the next transfer use a new (fresh) connection by force instead of
    /// trying to re-use an existing one. This option should be used with
    /// caution and only if you understand what it does as it may seriously
    /// impact performance.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_FRESH_CONNECT`.
    pub fn fresh_connect(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.fresh_connect(enable).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Make connection get closed at once after use.
    ///
    /// Makes libcurl explicitly close the connection when done with the
    /// transfer. Normally, libcurl keeps all connections alive when done with
    /// one transfer in case a succeeding one follows that can re-use them.
    /// This option should be used with caution and only if you understand what
    /// it does as it can seriously impact performance.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_FORBID_REUSE`.
    pub fn forbid_reuse(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.forbid_reuse(enable).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Timeout for the connect phase
    ///
    /// This is the maximum time that you allow the connection phase to the
    /// server to take. This only limits the connection phase, it has no impact
    /// once it has connected.
    ///
    /// By default this value is 300 seconds and corresponds to
    /// `CURLOPT_CONNECTTIMEOUT_MS`.
    pub fn connect_timeout(mut self, timeout: Duration) -> Result<Self, Error<C>> {
        self.easy.connect_timeout(timeout).map_err(Error::Curl)?;
        Ok(self)
    }

    // =========================================================================
    // Connection Options

    /// Set maximum time the request is allowed to take.
    ///
    /// Normally, name lookups can take a considerable time and limiting
    /// operations to less than a few minutes risk aborting perfectly normal
    /// operations.
    ///
    /// If libcurl is built to use the standard system name resolver, that
    /// portion of the transfer will still use full-second resolution for
    /// timeouts with a minimum timeout allowed of one second.
    ///
    /// In unix-like systems, this might cause signals to be used unless
    /// `nosignal` is set.
    ///
    /// Since this puts a hard limit for how long a request is allowed to
    /// take, it has limited use in dynamic use cases with varying transfer
    /// times. You are then advised to explore `low_speed_limit`,
    /// `low_speed_time` or using `progress_function` to implement your own
    /// timeout logic.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_TIMEOUT_MS`.
    pub fn timeout(mut self, timeout: Duration) -> Result<Self, Error<C>> {
        self.easy.timeout(timeout).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set preferred HTTP version.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_HTTP_VERSION`.
    pub fn http_version(mut self, version: HttpVersion) -> Result<Self, Error<C>> {
        self.easy.http_version(version).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set preferred TLS/SSL version.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_SSLVERSION`.
    pub fn ssl_version(mut self, version: SslVersion) -> Result<Self, Error<C>> {
        self.easy.ssl_version(version).map_err(Error::Curl)?;
        Ok(self)
    }

    // =========================================================================
    // Behavior options

    /// Configures this handle to have verbose output to help debug protocol
    /// information.
    ///
    /// By default output goes to stderr, but the `stderr` function on this type
    /// can configure that. You can also use the `debug_function` method to get
    /// all protocol data sent and received.
    ///
    /// By default, this option is `false`.
    pub fn verbose(mut self, verbose: bool) -> Result<Self, Error<C>> {
        self.easy.verbose(verbose).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Indicates whether header information is streamed to the output body of
    /// this request.
    ///
    /// This option is only relevant for protocols which have header metadata
    /// (like http or ftp). It's not generally possible to extract headers
    /// from the body if using this method, that use case should be intended for
    /// the `header_function` method.
    ///
    /// To set HTTP headers, use the `http_header` method.
    ///
    /// By default, this option is `false` and corresponds to
    /// `CURLOPT_HEADER`.
    pub fn show_header(mut self, show: bool) -> Result<Self, Error<C>> {
        self.easy.show_header(show).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Indicates whether a progress meter will be shown for requests done with
    /// this handle.
    ///
    /// This will also prevent the `progress_function` from being called.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_NOPROGRESS`.
    pub fn progress(mut self, progress: bool) -> Result<Self, Error<C>> {
        self.easy.progress(progress).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Specify the preferred receive buffer size, in bytes.
    ///
    /// This is treated as a request, not an order, and the main point of this
    /// is that the write callback may get called more often with smaller
    /// chunks.
    ///
    /// By default this option is the maximum write size and corresopnds to
    /// `CURLOPT_BUFFERSIZE`.
    pub fn download_buffer_size(mut self, size: usize) -> Result<Self, Error<C>> {
        self.easy.buffer_size(size).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Specify the preferred send buffer size, in bytes.
    ///
    /// This is treated as a request, not an order, and the main point of this
    /// is that the read callback may get called more often with smaller
    /// chunks.
    ///
    /// The upload buffer size is by default 64 kilobytes.
    pub fn upload_buffer_size(mut self, size: usize) -> Result<Self, Error<C>> {
        self.easy.upload_buffer_size(size).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Specify the preferred receive buffer size, in bytes.
    ///
    /// This is treated as a request, not an order, and the main point of this
    /// is that the write callback may get called more often with smaller
    /// chunks.
    ///
    /// By default this option is the maximum write size and corresopnds to
    /// `CURLOPT_BUFFERSIZE`.
    pub fn buffer_size(mut self, size: usize) -> Result<Self, Error<C>> {
        self.easy.buffer_size(size).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Re-initializes this handle to the default values.
    ///
    /// This puts the handle to the same state as it was in when it was just
    /// created. This does, however, keep live connections, the session id
    /// cache, the dns cache, and cookies.
    pub fn reset(&mut self) {
        self.easy.reset()
    }

    /// Provides the URL which this handle will work with.
    ///
    /// The string provided must be URL-encoded with the format:
    ///
    /// ```text
    /// scheme://host:port/path
    /// ```
    ///
    /// The syntax is not validated as part of this function and that is
    /// deferred until later.
    ///
    /// By default this option is not set and `perform` will not work until it
    /// is set. This option corresponds to `CURLOPT_URL`.
    pub fn url(mut self, url: &str) -> Result<Self, Error<C>> {
        self.easy.url(url).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set a custom request string
    ///
    /// Specifies that a custom request will be made (e.g. a custom HTTP
    /// method). This does not change how libcurl performs internally, just
    /// changes the string sent to the server.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_CUSTOMREQUEST`.
    pub fn custom_request(mut self, request: &str) -> Result<Self, Error<C>> {
        self.easy.custom_request(request).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Get the modification time of the remote resource
    ///
    /// If true, libcurl will attempt to get the modification time of the
    /// remote document in this operation. This requires that the remote server
    /// sends the time or replies to a time querying command. The `filetime`
    /// function can be used after a transfer to extract the received time (if
    /// any).
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_FILETIME`
    pub fn fetch_filetime(mut self, fetch: bool) -> Result<Self, Error<C>> {
        self.easy.fetch_filetime(fetch).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Indicate whether to download the request without getting the body
    ///
    /// This is useful, for example, for doing a HEAD request.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_NOBODY`.
    pub fn nobody(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.nobody(enable).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Set the size of the input file to send off.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_INFILESIZE_LARGE`.
    pub fn in_filesize(mut self, size: u64) -> Result<Self, Error<C>> {
        self.easy.in_filesize(size).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Enable or disable data upload.
    ///
    /// This means that a PUT request will be made for HTTP and probably wants
    /// to be combined with the read callback as well as the `in_filesize`
    /// method.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_UPLOAD`.
    pub fn upload(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.upload(enable).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Configure the maximum file size to download.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_MAXFILESIZE_LARGE`.
    pub fn max_filesize(mut self, size: u64) -> Result<Self, Error<C>> {
        self.easy.max_filesize(size).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Selects a condition for a time request.
    ///
    /// This value indicates how the `time_value` option is interpreted.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_TIMECONDITION`.
    pub fn time_condition(mut self, cond: TimeCondition) -> Result<Self, Error<C>> {
        self.easy.time_condition(cond).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Sets the time value for a conditional request.
    ///
    /// The value here should be the number of seconds elapsed since January 1,
    /// 1970. To pass how to interpret this value, use `time_condition`.
    ///
    /// By default this option is not set and corresponds to
    /// `CURLOPT_TIMEVALUE`.
    pub fn time_value(mut self, val: i64) -> Result<Self, Error<C>> {
        self.easy.time_value(val).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Start a new cookie session
    ///
    /// Marks this as a new cookie "session". It will force libcurl to ignore
    /// all cookies it is about to load that are "session cookies" from the
    /// previous session. By default, libcurl always stores and loads all
    /// cookies, independent if they are session cookies or not. Session cookies
    /// are cookies without expiry date and they are meant to be alive and
    /// existing for this "session" only.
    ///
    /// By default this option is `false` and corresponds to
    /// `CURLOPT_COOKIESESSION`.
    pub fn cookie_session(mut self, session: bool) -> Result<Self, Error<C>> {
        self.easy.cookie_session(session).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Ask for a HTTP GET request.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_HTTPGET`.
    pub fn get(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.get(enable).map_err(Error::Curl)?;
        Ok(self)
    }

    /// Make an HTTP POST request.
    ///
    /// This will also make the library use the
    /// `Content-Type: application/x-www-form-urlencoded` header.
    ///
    /// POST data can be specified through `post_fields` or by specifying a read
    /// function.
    ///
    /// By default this option is `false` and corresponds to `CURLOPT_POST`.
    pub fn post(mut self, enable: bool) -> Result<Self, Error<C>> {
        self.easy.post(enable).map_err(Error::Curl)?;
        Ok(self)
    }
}

/// The AsyncPerform struct is the result when calling nonblocking() function to signify the end of the builder.
/// The main job of this is to perform the Curl in nonblocking fashion.
pub struct AsyncPerform<C, A>
where
    C: Handler + Debug + Send + 'static,
    A: Actor<C>,
{
    /// This is the the actor handler that can be cloned to be able to handle multiple request sender
    /// and a single consumer that is spawned in the background upon creation of this object to be able to achieve
    /// non-blocking I/O during curl perform.
    actor: A,
    /// The `Easy2<C>` is the Easy2 from curl-rust crate wrapped in this struct to be able to do
    /// asynchronous task during perform operation.
    easy: Easy2<C>,
}

impl<C, A> AsyncPerform<C, A>
where
    C: ExtendedHandler + Debug + Send,
    A: Actor<C>,
{
    /// This will send the request asynchronously,
    /// and return the underlying [`Easy2<C>`](https://docs.rs/curl/latest/curl/easy/struct.Easy2.html) useful if you
    /// want to decide how to transform the response yourself.
    ///
    /// This becomes a non-blocking I/O since the actual perform operation is done
    /// at the actor side using Curl-Multi.
    pub async fn send_request(self) -> Result<Easy2<C>, Error<C>> {
        self.actor.send_request(self.easy).await.map_err(|e| {
            trace!("{:?}", e);
            Error::Perform(e)
        })
    }

    /// This will perform the curl operation asynchronously.
    pub async fn perform(self) -> Result<Response<Option<Vec<u8>>>, Error<C>> {
        let easy = self.send_request().await?;

        let (data, headers) = easy.get_ref().get_response_body_and_headers();
        let status_code = easy.response_code().map_err(|e| {
            trace!("{:?}", e);
            Error::Curl(e)
        })? as u16;

        let response_header = if let Some(response_header) = headers {
            response_header
        } else {
            let mut response_header = easy
                .content_type()
                .map_err(|e| {
                    trace!("{:?}", e);
                    Error::Curl(e)
                })?
                .map(|content_type| {
                    Ok(vec![(
                        CONTENT_TYPE,
                        HeaderValue::from_str(content_type).map_err(|err| {
                            trace!("{:?}", err);
                            Error::Http(err.to_string())
                        })?,
                    )]
                    .into_iter()
                    .collect::<HeaderMap>())
                })
                .transpose()?
                .unwrap_or_else(HeaderMap::new);

            let content_length = easy.content_length_download().map_err(|e| {
                trace!("{:?}", e);
                Error::Curl(e)
            })?;

            response_header.insert(
                CONTENT_LENGTH,
                HeaderValue::from_str(content_length.to_string().as_str()).map_err(|err| {
                    trace!("{:?}", err);
                    Error::Http(err.to_string())
                })?,
            );

            response_header
        };

        let mut response = Response::builder();
        for (name, value) in &response_header {
            response = response.header(name, value);
        }

        response = response.status(status_code);

        response.body(data).map_err(|e| Error::Http(e.to_string()))
    }
}

/// The SyncPerform struct is the result when calling blocking() function to signify the end of the builder.
/// The main job of this is to perform the Curl in blocking fashion.
pub struct SyncPerform<C>
where
    C: Handler + Debug + Send + 'static,
{
    easy: Easy2<C>,
}

impl<C> SyncPerform<C>
where
    C: ExtendedHandler + Debug + Send,
{
    /// This will send the request synchronously,
    /// and return the underlying [`Easy2<C>`](https://docs.rs/curl/latest/curl/easy/struct.Easy2.html) useful if you
    /// want to decide how to transform the response yourself.
    pub fn send_request(self) -> Result<Easy2<C>, Error<C>> {
        self.easy.perform().map_err(|e| {
            trace!("{:?}", e);
            Error::Perform(async_curl::error::Error::Curl(e))
        })?;

        Ok(self.easy)
    }

    /// This will perform the curl operation synchronously.
    pub fn perform(self) -> Result<Response<Option<Vec<u8>>>, Error<C>> {
        let easy = self.send_request()?;

        let (data, headers) = easy.get_ref().get_response_body_and_headers();
        let status_code = easy.response_code().map_err(|e| {
            trace!("{:?}", e);
            Error::Curl(e)
        })? as u16;

        let response_header = if let Some(response_header) = headers {
            response_header
        } else {
            let mut response_header = easy
                .content_type()
                .map_err(|e| {
                    trace!("{:?}", e);
                    Error::Curl(e)
                })?
                .map(|content_type| {
                    Ok(vec![(
                        CONTENT_TYPE,
                        HeaderValue::from_str(content_type).map_err(|err| {
                            trace!("{:?}", err);
                            Error::Http(err.to_string())
                        })?,
                    )]
                    .into_iter()
                    .collect::<HeaderMap>())
                })
                .transpose()?
                .unwrap_or_else(HeaderMap::new);

            let content_length = easy.content_length_download().map_err(|e| {
                trace!("{:?}", e);
                Error::Curl(e)
            })?;

            response_header.insert(
                CONTENT_LENGTH,
                HeaderValue::from_str(content_length.to_string().as_str()).map_err(|err| {
                    trace!("{:?}", err);
                    Error::Http(err.to_string())
                })?,
            );

            response_header
        };

        let mut response = Response::builder();
        for (name, value) in &response_header {
            response = response.header(name, value);
        }

        response = response.status(status_code);

        response.body(data).map_err(|e| Error::Http(e.to_string()))
    }
}

/// A strong type unit when setting download speed and upload speed
/// in Mega bits per second.
#[derive(Deref)]
pub struct Mbps(u32);
impl From<u32> for Mbps {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// A strong type unit when setting download speed and upload speed
/// in bytes per second.
#[derive(Deref)]
pub struct Bps(u64);

impl From<u64> for Bps {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Mbps> for Bps {
    fn from(value: Mbps) -> Self {
        Self::from((*value * 125_000) as u64)
    }
}

/// A strong type unit when offsetting especially in resuming download
/// or upload.
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

/// The purpose of this trait is to be able to accept
/// request body with Option<Vec<u8>> or Vec<u8>
pub trait CurlBodyRequest {
    fn get_bytes(&self) -> Option<&Vec<u8>>;
}

impl CurlBodyRequest for Vec<u8> {
    fn get_bytes(&self) -> Option<&Vec<u8>> {
        if self.is_empty() {
            None
        } else {
            Some(self)
        }
    }
}

impl CurlBodyRequest for Option<Vec<u8>> {
    fn get_bytes(&self) -> Option<&Vec<u8>> {
        self.as_ref()
    }
}
