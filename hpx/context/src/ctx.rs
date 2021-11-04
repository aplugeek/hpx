use crate::ContextState;
use hpx_app::Config;
use hpx_route::Route;
use hpx_sampling::{random_set, DEFAULT_RESERVOIR_SIZE};
use hpx_tracing::{start_tracing, Tracing};
use hyper::client::{Client, HttpConnector, ResponseFuture};
use hyper::http::Request;
use hyper::Body;
use mick_jaeger::TracesIn;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub struct GTX {
    pub inner: Arc<Context>,
}

pub struct Context {
    route: RwLock<Arc<Route>>,
    h1_client: Client<HttpConnector, Body>,
    trace_in: Option<Arc<TracesIn>>,
    state: ContextState,
}

impl Context {
    pub async fn with_config(conf: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let mut connector = HttpConnector::new();
        connector.set_connect_timeout(Some(Duration::from_secs(conf.connect_timeout as u64)));
        connector.set_nodelay(true);
        let mut ctx = Self {
            route: RwLock::default(),
            h1_client: Client::builder()
                .pool_idle_timeout(Some(Duration::from_secs(5)))
                .build(connector),
            trace_in: None,
            state: ContextState {
                sampling: random_set(DEFAULT_RESERVOIR_SIZE, conf.sampling_percentage),
                counter: AtomicUsize::new(0),
            },
        };
        if let Some(udp) = conf.tracing_udp {
            let trace_in = start_tracing(udp.as_str(), conf.env_code.as_str()).await?;
            ctx.trace_in = Some(trace_in);
        }

        Ok(ctx)
    }

    pub fn get_state(&self) -> &ContextState {
        &self.state
    }
}

pub trait Forward {
    fn get_route(&self) -> Arc<Route>;
    fn reload_route(&self, route: Route);
    fn send_tracing(&self, trace: Tracing, status_code: u16, target: &str, msg: &str);
    fn forward_to(&self, req: Request<Body>) -> ResponseFuture;
}

impl Forward for Arc<Context> {
    fn get_route(&self) -> Arc<Route> {
        let lock = self.route.read().unwrap();
        (*lock).clone()
    }

    fn reload_route(&self, route: Route) {
        let mut lock = self.route.write().unwrap();
        *lock = Arc::new(route);
    }

    fn send_tracing(&self, trace: Tracing, status_code: u16, target: &str, msg: &str) {
        if let Some(trace_in) = &self.trace_in {
            let mut _span = trace_in.span_with_id_and_parent(
                trace.trace_id,
                trace.span_id,
                trace.parent_id,
                target,
            );
            _span.add_int_tag("status_code", status_code as i64);
            if msg.len() > 0 {
                _span.log().with_string("message", msg);
            }
        }
    }

    fn forward_to(&self, req: Request<Body>) -> ResponseFuture {
        self.h1_client.request(req)
    }
}
