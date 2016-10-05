use std::fmt::Write;

use bytes::buf::{BlockBuf, Fmt};
use tokio_proto::pipeline::Frame;
use tokio_proto::Serialize;

use ::header;
use ::http::ResponseHead;

pub struct Response;

impl Serialize for Response {
    type In = Frame<ResponseHead, (), ::Error>;

    fn serialize(&mut self, frame: Self::In, buf: &mut BlockBuf) {
        match frame {
            Frame::Message(mut head) => {
                trace!("writing head: {:?}", head);

                if !head.headers.has::<header::Date>() {
                    head.headers.set(header::Date(header::HttpDate(::time::now_utc())));
                }

                let mut is_chunked = true;
                //let mut body = Encoder::chunked();
                if let Some(cl) = head.headers.get::<header::ContentLength>() {
                    //body = Encoder::length(**cl);
                    is_chunked = false
                }

                if is_chunked {
                    let encodings = match head.headers.get_mut::<header::TransferEncoding>() {
                        Some(&mut header::TransferEncoding(ref mut encodings)) => {
                            if encodings.last() != Some(&header::Encoding::Chunked) {
                                encodings.push(header::Encoding::Chunked);
                            }
                            false
                        },
                        None => true
                    };

                    if encodings {
                        head.headers.set(header::TransferEncoding(vec![header::Encoding::Chunked]));
                    }
                }


                //let init_cap = 30 + head.headers.len() * AVERAGE_HEADER_SIZE;
                //dst.reserve(init_cap);
                debug!("writing {:#?}", head.headers);
                let _ = write!(Fmt(buf), "{} {}\r\n{}\r\n", head.version, head.subject, head.headers);
                /*
                if head.version == ::HttpVersion::Http11 && head.subject == ::StatusCode::Ok {
                    extend(dst, b"HTTP/1.1 200 OK\r\n");
                    let _ = write!(FastWrite(dst), "{}\r\n", head.headers);
                } else {
                    let _ = write!(FastWrite(dst), "{} {}\r\n{}\r\n", head.version, head.subject, head.headers);
                }
                */
                //body
            },
            _ => unimplemented!("Frame::*")
        }

    }
}

