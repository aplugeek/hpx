//! The main entrypoint for the proxy.

#[macro_use]
extern crate log;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use env_logger::Env;
use futures::FutureExt;
use futures::{join, select};
use hpx_app::{App, Config};
use hpx_context::ctx::{Forward, GTX};
use hpx_context::Context;
use hpx_forward::proxy;
use hpx_register::register_server;
use hpx_signal as signal;
use hyper::http::Request;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Server};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::mpsc;

mod rt;

fn main() {
    let app = App::from_args();
    let App {
        worker_thread,
        ref level,
        port,
        ..
    } = app;
    env_logger::from_env(Env::default().default_filter_or(level)).init();
    rt::build(worker_thread).block_on(async move {
        let gtx = GTX {
            inner: Arc::new(
                Context::with_config(Config::init())
                    .await
                    .expect("Context init error"),
            ),
        };
        let static_ctx: &'static GTX = unsafe { std::mem::transmute(&gtx) };
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel::<()>();
        let socket_addr = &SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), port);
        let server = Server::bind(&socket_addr)
            .http1_keepalive(true)
            .tcp_nodelay(true)
            .serve(make_service_fn(move |_| async move {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| async move {
                    let ctx = static_ctx.inner.clone();
                    let route = ctx.get_route();
                    proxy(ctx, route, req).await
                }))
            }))
            .with_graceful_shutdown(async {
                shutdown_rx.recv().await;
                info!("Server has graceful shutdown!")
            });
        let side_future = async move {
            let route_serve = async move {
                let bind = register_server(static_ctx, "/tmp/hpx/hpx.sock").await;
                if let Err(e) = bind {
                    error!("bind register server error: {:?}", e);
                    std::process::exit(1)
                }
            };
            let signal = async move {
                signal::shutdown().await;
                shutdown_tx.send(())
            };
            join!(route_serve, signal);
        };

        select!(_=Box::pin(server.fuse())=>(), _=Box::pin(side_future.fuse())=>());
    });
}
