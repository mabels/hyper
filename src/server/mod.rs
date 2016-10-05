//! HTTP Server
//!
//! A `Server` is created to listen on a port, parse HTTP requests, and hand
//! them off to a `Handler`.
use std::cell::RefCell;
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use bytes::buf::BlockBuf;

use futures::{Future, Async, Map};
use futures::stream::{Stream, Receiver};

use tokio::reactor::Core;
use tokio_proto::server::listen as tokio_listen;
use tokio_proto::pipeline;
use tokio_proto::{Framed, Message};
pub use tokio_service::{NewService, Service};

pub use self::request::Request;
pub use self::response::Response;

use self::request::Parser;
use self::response::Serializer;

use http;

pub use net::{Accept, HttpListener};
use net::{HttpStream, Transport};
/*
pub use net::{Accept, HttpListener, HttpsListener};
use net::{SslServer, Transport};
*/


mod request;
mod response;
//mod txn;

/// A Server that can accept incoming network requests.
#[derive(Debug)]
pub struct Server<A> {
    //listeners: Vec<A>,
    _marker: PhantomData<A>,
    addr: SocketAddr,
    keep_alive: bool,
    idle_timeout: Option<Duration>,
    max_sockets: usize,
}

impl<A: Accept> Server<A> {
    /*
    /// Creates a new Server from one or more Listeners.
    ///
    /// Panics if listeners is an empty iterator.
    pub fn new<I: IntoIterator<Item = A>>(listeners: I) -> Server<A> {
        let listeners = listeners.into_iter().collect();

        Server {
            listeners: listeners,
            keep_alive: true,
            idle_timeout: Some(Duration::from_secs(10)),
            max_sockets: 4096,
        }
    }
    */

    /// Enables or disables HTTP keep-alive.
    ///
    /// Default is true.
    pub fn keep_alive(mut self, val: bool) -> Server<A> {
        self.keep_alive = val;
        self
    }

    /// Sets how long an idle connection will be kept before closing.
    ///
    /// Default is 10 seconds.
    pub fn idle_timeout(mut self, val: Option<Duration>) -> Server<A> {
        self.idle_timeout = val;
        self
    }

    /// Sets the maximum open sockets for this Server.
    ///
    /// Default is 4096, but most servers can handle much more than this.
    pub fn max_sockets(mut self, val: usize) -> Server<A> {
        self.max_sockets = val;
        self
    }
}

impl Server<HttpListener> { //<H: HandlerFactory<<HttpListener as Accept>::Output>> Server<HttpListener, H> {
    /// Creates a new HTTP server config listening on the provided address.
    pub fn http(addr: &SocketAddr) -> ::Result<Server<HttpListener>> {
        Ok(Server {
            _marker: PhantomData,
            addr: addr.clone(),
            keep_alive: true,
            idle_timeout: Some(Duration::from_secs(10)),
            max_sockets: 4096,
        })
    }
}


/*
impl<S: SslServer> Server<HttpsListener<S>> {
    /// Creates a new server config that will handle `HttpStream`s over SSL.
    ///
    /// You can use any SSL implementation, as long as it implements `hyper::net::Ssl`.
    pub fn https(addr: &SocketAddr, ssl: S) -> ::Result<Server<HttpsListener<S>>> {
        HttpsListener::new(addr, ssl)
            .map(Server::new)
            .map_err(From::from)
    }
}
*/


impl/*<A: Accept>*/ Server<HttpListener> {
    /// Binds to a socket and starts handling connections.
    pub fn handle<H>(mut self, factory: H) -> ::Result<(Listening, ServerLoop<H>)>
    where H: NewService<Request=Request, Response=Response, Error=::Error> + Send + 'static {
        let mut core = try!(Core::new());
        let handle = core.handle();
        let _ = try!(tokio_listen(&handle, self.addr, move |sock| {
            let service = HttpService { inner: try!(factory.new_service()) };
            let framed = Framed::new(
                sock,
                Parser,
                Serializer,
                BlockBuf::default(),
                BlockBuf::default()
            );
            pipeline::Server::new(service, framed)
        }));
        core.run(::futures::empty::<(), ()>()).unwrap();

        unimplemented!()
    }
}

/// A configured `Server` ready to run.
pub struct ServerLoop<H> {
    inner: Option<(Core, Box<Future<Item=(), Error=()>>)>,
    _marker: PhantomData<H>
}

impl<H> fmt::Debug for ServerLoop<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("ServerLoop")
    }
}

impl<H> ServerLoop<H> {
    /// Runs the server forever in this loop.
    ///
    /// This will block the current thread.
    pub fn run(self) {
        // drop will take care of it.
    }
}

impl<H> Drop for ServerLoop<H> {
    fn drop(&mut self) {
        self.inner.take().map(|(mut loop_, work)| {
            let _ = loop_.run(work);
        });
    }
}

/// A handle of the running server.
pub struct Listening {
    addrs: Vec<SocketAddr>,
    //shutdown: Sender<()>,
}

impl fmt::Debug for Listening {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Listening")
            .field("addrs", &self.addrs)
            .finish()
    }
}

impl fmt::Display for Listening {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, addr) in self.addrs().iter().enumerate() {
            if i > 1 {
                try!(f.write_str(", "));
            }
            try!(fmt::Display::fmt(addr, f));
        }
        Ok(())
    }
}

impl Listening {
    /// The addresses this server is listening on.
    pub fn addrs(&self) -> &[SocketAddr] {
        &self.addrs
    }

    /// Stop the server from listening to its socket address.
    pub fn close(self) {
        debug!("closing server {}", self);
        //let _ = self.shutdown.send(());
    }
}

struct HttpService<T> {
        inner: T,
}

impl<T> Service for HttpService<T>
    where T: Service<Request=Request, Response=Response, Error=::Error>,
{
    type Request = Request;
    type Response = Message<Response, Receiver<(), ::Error>>;
    type Error = ::Error;
    type Future = Map<T::Future, fn(Response) -> Message<Response, Receiver<(), ::Error>>>;

    fn call(&self, req: Request) -> Self::Future {
        self.inner.call(req).map(Message::WithoutBody)
    }

    fn poll_ready(&self) -> Async<()> {
        self.inner.poll_ready()
    }
}
