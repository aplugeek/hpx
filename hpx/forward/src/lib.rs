#[macro_use]
extern crate log;

use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::task::Poll;

use hpx_error::not_found;
use hpx_route::{Route, Servant, Server};
use hyper::client::ResponseFuture;
use hyper::http::{Request, Response, StatusCode, Uri};
use hyper::Body;

mod handle;
use crate::to_response;
pub use handle::*;
use hpx_context::ctx::Forward;
use hpx_context::Context;

struct Respond {
    target: Option<String>,
    inner: Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send>>,
}

enum RespondKind {
    Forward(Arc<Context>, Arc<Route>, Request<Body>),
    Upgrade(Arc<Context>, Arc<Route>, Request<Body>),
}

impl Respond {
    pub fn new(inner: ResponseFuture, target: &str) -> Self {
        Self {
            inner: Box::pin(inner),
            target: Some(target.to_owned()),
        }
    }
}

impl Future for Respond {
    type Output = Response<Body>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.inner).poll(cx) {
            Poll::Ready(result) => match result {
                Ok(resp) => Poll::Ready(resp),
                Err(e) => {
                    error!("Forward to {:?} error: {:?}", self.target, e);
                    Poll::Ready(to_response(
                        StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                        e.to_string(),
                    ))
                }
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Respond {
    pub fn from_kind(kind: RespondKind) -> Self {
        match kind {
            RespondKind::Forward(ctx, route, mut req) => {
                let servant_idx = match route.rmap.get(req.uri().path()) {
                    Some(index) => index,
                    None => match route.rtrie.get_ancestor_value(req.uri().path()) {
                        Some(index) => index,
                        None => {
                            return Respond {
                                target: None,
                                inner: Box::pin(futures::future::ok(not_found())),
                            };
                        }
                    },
                };
                let s: &Servant = &route.servant[*servant_idx];
                let count = s.state.count.fetch_add(1, Ordering::SeqCst);
                let server: &Server = &s.servers[count % (s.servers.len())];
                let forward_uri = match req.uri().query() {
                    Some(query) => format!("http://{}{}?{}", server.addr, req.uri().path(), query),
                    None => format!("http://{}{}", server.addr, req.uri().path()),
                };
                *req.uri_mut() = Uri::from_str(forward_uri.as_str()).unwrap();
                Respond {
                    target: Some(s.name.clone()),
                    inner: Box::pin(ctx.forward_to(req)),
                }
            }
            RespondKind::Upgrade(_, _, _) => {
                unreachable!()
            }
        }
    }
}
