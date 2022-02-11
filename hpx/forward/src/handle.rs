use crate::{Respond, RespondKind};

#[macro_use]
use hpx_middleware::middleware;
use hpx_middleware::middleware::{
    is_sampling, parse_trace, sampling_rate_ctl, with_body_size_limit, with_print, with_trace,
};
use hpx_route::Route;
use hyper::http::header::UPGRADE;
use hyper::http::{HeaderMap, HeaderValue, Request, Response, StatusCode};
use hyper::Body;

use std::num::{NonZeroU128, NonZeroU64};

use hpx_context::ctx::{Forward, SendTrace};
use hpx_context::Context;
use hpx_tracing::{set_tracing_header, Tracing, X_PARENT_ID, X_SAMPLING, X_SPAN_ID, X_TRACE_ID};
use serde::de::Unexpected::Bytes;
use std::sync::Arc;

pub async fn proxy(
    ctx: Arc<Context>,
    route: Arc<Route>,
    mut req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let (mut_req, ctx_ref) = (&mut req, &ctx);
    if let Err(e) = middleware!(
        mut_req,
        ctx_ref,
        with_print,
        with_body_size_limit,
        sampling_rate_ctl,
        with_trace
    ) {
        return Ok(to_response(e.code, e.message));
    }
    let (sampling, trace) = (is_sampling(mut_req), parse_trace(mut_req));
    let respond = Respond::from_kind(to_respond_kind(ctx.clone(), route, req));

    Ok(trace_respond(ctx, sampling, trace, respond).await?)
}

async fn trace_respond(
    ctx: Arc<Context>,
    sampling: bool,
    trace: Tracing,
    respond: Respond,
) -> Result<Response<Body>, hyper::Error> {
    let target = respond.target.clone();
    let mut response = respond.await;
    if !sampling {
        return Ok(response);
    }
    let mut body_bytes = bytes::Bytes::new();
    if !response.status().is_success() {
        let (part, body) = response.into_parts();
        body_bytes = hyper::body::to_bytes(body).await?;
        response = Response::from_parts(part, Body::from(body_bytes.clone()));
    }
    let status_code = response.status_mut().as_u16();
    if let Some(t) = target {
        tokio::spawn(async move {
            send_tracing(ctx, trace, t.as_str(), status_code, body_bytes.to_vec()).await;
        });
    }
    set_tracing_header(trace, sampling, response.headers_mut());

    Ok(response)
}

fn to_respond_kind(ctx: Arc<Context>, route: Arc<Route>, req: Request<Body>) -> RespondKind {
    match req.headers().get(UPGRADE) {
        Some(_) => RespondKind::Upgrade(ctx, route, req),
        None => RespondKind::Forward(ctx, route, req),
    }
}

pub(crate) fn to_response(code: u16, message: String) -> Response<Body> {
    let mut respond = Response::new(Body::from(message.to_string()));
    *respond.status_mut() = StatusCode::from_u16(code).unwrap_or(StatusCode::default());
    respond
}

async fn send_tracing(
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
