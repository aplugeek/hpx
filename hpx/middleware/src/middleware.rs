use hpx_context::Context;
use hpx_error::error::AppResponseError;
use hpx_tracing::{set_tracing_header, Tracing, X_PARENT_ID, X_SAMPLING, X_SPAN_ID, X_TRACE_ID};
use hyper::http::header::{HeaderName, CONTENT_TYPE};
use hyper::http::{HeaderMap, HeaderValue, Request};
use hyper::Body;
use rand::Rng;
use std::num::{NonZeroU128, NonZeroU64};
use std::sync::Arc;

#[macro_export]
macro_rules! middleware {
    ($req:tt,$ctx:tt,$b:tt) => {
        $b($ctx,$req)
    };
    ($req:tt,$ctx:tt,$a:tt,$($b:tt),*) => {
        match $a($ctx,$req) {
            Ok(_) => {
                middleware!($req,$ctx,$($b),*)
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}

pub const X_REQUEST_ID: &'static str = "x-request-id";

pub fn with_trace(_: &Arc<Context>, req: &mut Request<Body>) -> Result<(), AppResponseError> {
    set_tracing_header(parse_trace(req), is_sampling(req), req.headers_mut());
    Ok(())
}

pub fn parse_trace(req: &mut Request<Body>) -> Tracing {
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

pub fn with_print(_: &Arc<Context>, req: &mut Request<Body>) -> Result<(), AppResponseError> {
    debug!("request incoming:{:?}", req);
    Ok(())
}

pub fn with_body_size_limit(
    _: &Arc<Context>,
    _: &mut Request<Body>,
) -> Result<(), AppResponseError> {
    Ok(())
}

pub fn sampling_rate_ctl(
    ctx: &Arc<Context>,
    req: &mut Request<Body>,
) -> Result<(), AppResponseError> {
    if let Some(content_type) = req.headers().get(CONTENT_TYPE) {
        if content_type.eq("application/grpc") {
            return Ok(());
        }
    }
    if let None = req.headers().get(X_TRACE_ID) {
        let state = ctx.get_state();
        let c = rand::thread_rng().gen_range(0..100);
        if state.should_sampling(c as usize) {
            req.headers_mut()
                .insert(X_SAMPLING, HeaderValue::from_static("true"));
        }
    }

    Ok(())
}

pub fn is_sampling(req: &mut Request<Body>) -> bool {
    if let Some(_) = req.headers().get(X_SAMPLING) {
        return true;
    }
    false
}
