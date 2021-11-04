use hpx_context::Context;
use hpx_error::error::AppResponseError;
use hpx_tracing::{X_SAMPLING, X_TRACE_ID};
use hyper::http::header::HeaderName;
use hyper::http::{HeaderValue, Request};
use hyper::Body;
use rand::Rng;
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
    let trace_id = uuid::Uuid::new_v4().to_string();
    req.headers_mut().insert(
        HeaderName::from_static(X_REQUEST_ID),
        HeaderValue::from_bytes(trace_id.as_bytes()).unwrap(),
    );
    Ok(())
}

pub fn with_print(_: &Arc<Context>, req: &mut Request<Body>) -> Result<(), AppResponseError> {
    debug!("request incoming:{:?}", req);
    Ok(())
}

pub fn sampling_rate_ctl(
    ctx: &Arc<Context>,
    req: &mut Request<Body>,
) -> Result<(), AppResponseError> {
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
