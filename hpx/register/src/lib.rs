#[macro_use]
extern crate log;

use crate::unix::SocketIncoming;
use hpx_context::ctx::{Forward, GTX};
use hpx_error::{bad_request, not_found, status_ok};
use hpx_route::{Route, RouteEndpoint, RoutePath};
use hyper::body::Buf;
use hyper::http::header::CONTENT_TYPE;
use hyper::http::{Method, Request, Response, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::Body;
use hyper::Server;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::net::UnixListener;

mod unix;

#[derive(Serialize, Deserialize, Debug)]
struct ServiceRoute {
    #[serde(rename = "servant")]
    pub servant: String,
    #[serde(rename = "routes")]
    pub routes: Vec<RoutePath>,
    #[serde(rename = "endpoints")]
    pub endpoints: Vec<String>,
}

pub async fn register_server(ctx: &'static GTX, uds: &str) -> std::io::Result<()> {
    if Path::new(uds).exists() {
        std::fs::remove_file(uds)?
    }
    let unix_listener = UnixListener::bind(uds)?;
    let incoming = SocketIncoming::from_listener(unix_listener);
    let server = Server::builder(incoming);
    server
        .serve(make_service_fn(|_| async move {
            Ok::<_, hyper::Error>(service_fn(move |req| serve_http(ctx, req)))
        }))
        .await
        .unwrap();

    Ok(())
}

pub async fn serve_http(
    ctx: &'static GTX,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/route/register") => {
            let body = hyper::body::aggregate(req.into_body()).await?;
            let service_routes: Vec<ServiceRoute> = match serde_json::from_reader(body.reader()) {
                Ok(s) => s,
                Err(e) => return Ok(bad_request(e.to_string())),
            };
            let mut rmap = HashMap::with_capacity(service_routes.len());
            service_routes.iter().for_each(|r| {
                rmap.insert(
                    r.servant.clone(),
                    RouteEndpoint {
                        routes: r.routes.clone(),
                        endpoints: r.endpoints.clone(),
                    },
                );
            });
            let route = Route::from_endpoints(&rmap).unwrap();
            ctx.inner.reload_route(route);
            info!("Reload hpx routes succeed");
            Ok(status_ok())
        }
        (&Method::GET, "/routes") => {
            let route = ctx.inner.get_route();
            let mut rst = HashMap::new();
            rst.insert("servant", route.servant.clone());
            let b = serde_json::to_string(&rst).unwrap_or(String::from("NULL"));
            let resp = Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "application/json; charset=UTF-8")
                .body(Body::from(b))
                .unwrap();
            Ok(resp)
        }
        _ => Ok(not_found()),
    }
}
