//! Server Responses
//!
//! These are responses sent by a `hyper::Server` to clients, after
//! receiving a request.

use bytes::buf::{BlockBuf};
use bytes::MutBuf;
use tokio_proto::pipeline::Frame;
use tokio_proto::Serialize;

use header;
use http;
use status::StatusCode;
use version;


/// The outgoing half for a Tcp connection, created by a `Server` and given to a `Handler`.
///
/// The default `StatusCode` for a `Response` is `200 OK`.
#[derive(Debug, Default)]
pub struct Response {
    head: http::MessageHead<StatusCode>,
    body: Option<&'static [u8]>,
}

impl Response {
    /// Create a new Response.
    #[inline]
    pub fn new() -> Response {
        Response::default()
    }

    pub fn header<H: ::header::Header>(mut self, header: H) -> Self {
        self.head.headers.set(header);
        self
    }

    pub fn body(mut self, buf: &'static [u8]) -> Self {
        self.body = Some(buf);
        self
    }

    /*
    /// The headers of this response.
    #[inline]
    pub fn headers(&self) -> &header::Headers { &self.head.headers }

    /// The status of this response.
    #[inline]
    pub fn status(&self) -> &StatusCode {
        &self.head.subject
    }

    /// The HTTP version of this response.
    #[inline]
    pub fn version(&self) -> &version::HttpVersion { &self.head.version }

    /// Get a mutable reference to the Headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut header::Headers { &mut self.head.headers }

    /// Get a mutable reference to the status.
    #[inline]
    pub fn set_status(&mut self, status: StatusCode) {
        self.head.subject = status;
    }
    */
}

pub struct Serializer;

impl Serialize for Serializer {
    type In = Frame<Response, (), ::Error>;

    fn serialize(&mut self, frame: Self::In, buf: &mut BlockBuf) {
        let frame = match frame {
            Frame::Message(mut res) => {
                if res.body.is_none() {
                    res.head.headers.set(::header::ContentLength(0));
                }
                ::http::h1::serialize::Response.serialize(Frame::Message(res.head.into()), buf);
                if let Some(body) = res.body {
                    buf.copy_from(body);
                }
            },
            _ => unimplemented!("Frame::*")
        };
    }
}
