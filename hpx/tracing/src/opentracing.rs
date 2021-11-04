use mick_jaeger::TracesIn;
use std::num::{NonZeroU128, NonZeroU64};
use std::sync::Arc;

pub const X_SAMPLING: &'static str = "x-sampling";
pub const X_TRACE_ID: &'static str = "x-trace-id";
pub const X_SPAN_ID: &'static str = "x-span-id";
pub const X_PARENT_ID: &'static str = "x-parent-id";

#[derive(Copy, Clone)]
pub struct Tracing {
    pub trace_id: NonZeroU128,
    pub span_id: NonZeroU64,
    pub parent_id: Option<NonZeroU64>,
}

pub async fn start_tracing(
    udp: &str,
    env_code: &str,
) -> Result<Arc<TracesIn>, Box<dyn std::error::Error>> {
    let (traces_in, mut traces_out) = mick_jaeger::init(mick_jaeger::Config {
        service_name: format!("HPX-{}", env_code),
    });

    let udp_socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
    udp_socket.connect(udp).await?;
    tokio::spawn(async move {
        loop {
            let buf = traces_out.next().await;
            udp_socket.send(&buf).await;
        }
    });

    return Ok(traces_in);
}
