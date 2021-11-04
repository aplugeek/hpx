use hyper::server::accept::Accept;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io;
use tokio::net::{UnixListener, UnixStream};

/// A stream of connections from binding to a socket.
#[pin_project]
#[derive(Debug)]
pub struct SocketIncoming {
    listener: UnixListener,
}

impl SocketIncoming {
    pub fn from_listener(listener: UnixListener) -> Self {
        Self { listener }
    }
}

impl Accept for SocketIncoming {
    type Conn = UnixStream;
    type Error = io::Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let conn = futures::ready!(self.listener.poll_accept(cx))?.0;
        Poll::Ready(Some(Ok(conn)))
    }
}
