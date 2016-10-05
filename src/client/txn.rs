use std::io;

use http;
use net::Transport;

use super::{request, response, Request, Response, Handler};

pub struct Handle<H> {
    handler: H,
}

impl<H> Handle<H> {
    pub fn new(handler: H) -> Handle<H> {
        Handle {
            handler: handler,
        }
    }
}

impl<H: Handler<T>, T: Transport> http::TransactionHandler<T> for Handle<H> {
    type Transaction = http::ClientTransaction;

    #[inline]
    fn ready(&mut self, txn: &mut http::Transaction<T, Self::Transaction>) {
        let mut outer = Transaction { inner: txn };
        self.handler.ready(&mut outer);
    }
}



pub struct Transaction<'a: 'b, 'b, T: Transport + 'a> {
    inner: &'b mut http::Transaction<'a, T, http::ClientTransaction>,
}

impl<'a: 'b, 'b, T: Transport + 'a> Transaction<'a, 'b, T> {
    #[inline]
    pub fn request(&mut self) -> Request {
        self.inner.incoming().map(request::new)
    }

    #[inline]
    pub fn response(&mut self) -> ::Result<Response> {
        self.inner.incoming().map(response::new)
    }

    #[inline]
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    #[inline]
    pub fn try_read(&mut self, buf: &mut [u8]) -> io::Result<Option<usize>> {
        self.inner.try_read(buf)
    }


    #[inline]
    pub fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.inner.write(data)
    }

    #[inline]
    pub fn try_write(&mut self, data: &[u8]) -> io::Result<Option<usize>> {
        self.inner.try_write(data)
    }

    #[inline]
    pub fn end(&mut self) {
        self.inner.end();
    }
}

impl<'a: 'b, 'b, T: Transport + 'a> io::Read for Transaction<'a, 'b, T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read(buf)
    }
}

impl<'a: 'b, 'b, T: Transport + 'a> io::Write for Transaction<'a, 'b, T> {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.write(data)
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!("Transaction.flush")
    }
}
