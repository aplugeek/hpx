use crate::{Respond, RespondKind};

#[macro_use]
use hpx_middleware::middleware;
use hpx_middleware::middleware::{sampling_rate_ctl, with_print, with_trace};
use hpx_route::Route;
use hyper::http::header::UPGRADE;
use hyper::http::{HeaderValue, Request, Response};
use hyper::Body;

use std::num::{NonZeroU128, NonZeroU64};

use hpx_context::ctx::Forward;
use hpx_context::Context;
use hpx_tracing::{Tracing, X_PARENT_ID, X_SAMPLING, X_SPAN_ID, X_TRACE_ID};
use serde::de::Unexpected::Bytes;
use std::sync::Arc;

pub async fn proxy(
    ctx: Arc<Context>,
    route: Arc<Route>,
    mut req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let (sreq, ctx_ref) = (&mut req, &ctx);
    if let Err(e) = middleware!(sreq, ctx_ref, with_print, sampling_rate_ctl) {
        let mut respond = Response::new(Body::from(e.message.clone()));
        *respond.status_mut() = e.status_code;
        return Ok(respond);
    }
    let (forward_uri, trace, sampling) = (
        sreq.uri().clone(),
        gen_open_tracing(sreq),
        is_sampling(sreq),
    );
    let origin_resp = Respond::from_kind(get_respond_kind(ctx.clone(), route, req));
    let target = origin_resp.target.clone();
    let mut response = origin_resp.await;
    let mut body_bytes = bytes::Bytes::new();
    if !response.status().is_success() {
        let (part, body) = response.into_parts();
        body_bytes = hyper::body::to_bytes(body).await?;
        response = Response::from_parts(part, Body::from(body_bytes.clone()));
    }
    let status_code = response.status().as_u16();
    trace!(
        "Forward to{:?}, status_code:{:?}",
        forward_uri,
        response.status().as_u16()
    );
    if let Some(t) = target {
        if sampling {
            tokio::spawn(async move {
                trace_respond(ctx, trace, t.as_str(), status_code, body_bytes.to_vec()).await;
            });
        }
    }
    set_tracing_header(trace, sampling, &mut response);

    Ok(response)
}

fn set_tracing_header(trace: Tracing, sampling: bool, resp: &mut Response<Body>) {
    resp.headers_mut().insert(
        X_TRACE_ID,
        HeaderValue::from_str(trace.trace_id.to_string().as_str()).unwrap(),
    );
    resp.headers_mut().insert(
        X_SPAN_ID,
        HeaderValue::from_str(trace.span_id.to_string().as_str()).unwrap(),
    );
    if sampling {
        resp.headers_mut()
            .insert(X_SAMPLING, HeaderValue::from_static("true"));
    }
}

fn gen_open_tracing(req: &mut Request<Body>) -> Tracing {
    let mut trace = Tracing {
        trace_id: NonZeroU128::new(rand::random()).unwrap(),
        span_id: NonZeroU64::new(rand::random()).unwrap(),
        parent_id: None,
    };

    if let Some(root) = req.headers().get(X_TRACE_ID) {
        trace.trace_id = root
            .to_str()
            .map_or(NonZeroU128::new(rand::random()).unwrap(), |v| {
                v.parse()
                    .unwrap_or(NonZeroU128::new(rand::random()).unwrap())
            });
    }
    if let Some(span) = req.headers().get(X_SPAN_ID) {
        trace.span_id = span
            .to_str()
            .map_or(NonZeroU64::new(rand::random()).unwrap(), |v| {
                v.parse()
                    .unwrap_or(NonZeroU64::new(rand::random()).unwrap())
            });
    }
    if let Some(parent) = req.headers().get(X_PARENT_ID) {
        trace.parent_id = parent.to_str().map_or(None, |v| {
            v.parse()
                .map_or(None, |v| Some(NonZeroU64::new(v).unwrap()))
        });
    }

    trace
}

fn is_sampling(req: &mut Request<Body>) -> bool {
    if let Some(_) = req.headers().get(X_SAMPLING) {
        return true;
    }
    false
}

fn get_respond_kind(ctx: Arc<Context>, route: Arc<Route>, req: Request<Body>) -> RespondKind {
    match req.headers().get(UPGRADE) {
        Some(_) => RespondKind::Upgrade(ctx, route, req),
        None => RespondKind::Forward(ctx, route, req),
    }
}

async fn trace_respond(
    ctx: Arc<Context>,
    trace: Tracing,
    target: &str,
    status_code: u16,
    buf: Vec<u8>,
) {
    unsafe {
        ctx.send_tracing(
            trace,
            status_code,
            target,
            String::from_utf8_unchecked(buf).as_str(),
        )
    }
}
