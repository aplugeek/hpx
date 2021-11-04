pub mod error;

use crate::error::AppResponseError;
use hyper::http::header::CONTENT_TYPE;
use hyper::http::{HeaderValue, Response, StatusCode};
use hyper::Body;

pub fn not_found() -> Response<Body> {
    let err = AppResponseError::from(
        StatusCode::NOT_FOUND.as_u16(),
        "route not found",
        StatusCode::NOT_FOUND,
    );
    let b = serde_json::to_string(&err).unwrap_or(String::from("route not found"));
    let mut resp = Response::new(Body::from(b));
    *resp.status_mut() = err.status_code;
    resp.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=UTF-8"),
    );
    resp
}

pub fn bad_request(err: String) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(err))
        .unwrap()
}

pub fn status_ok() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}
