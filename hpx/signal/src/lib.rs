//! Unix signal handling for the proxy binary.

#![deny(warnings, rust_2018_idioms)]
#![forbid(unsafe_code)]

#[macro_use]
extern crate log;
/// Returns a `Future` that completes when the proxy should start to shutdown.
pub async fn shutdown() {
    imp::shutdown().await
}

#[cfg(unix)]
mod imp {
    use tokio::signal::unix::{signal, SignalKind};

    pub(super) async fn shutdown() {
        tokio::select! {
            () = sig(SignalKind::interrupt(), "SIGINT") => {}
            () = sig(SignalKind::terminate(), "SIGTERM") => {}
        };
    }

    async fn sig(kind: SignalKind, name: &'static str) {
        signal(kind)
            .expect("Failed to register signal handler")
            .recv()
            .await;
        info!("hbx::signal received {}, starting shutdown", name);
    }
}

#[cfg(not(unix))]
mod imp {
    use tracing::info;

    pub(super) async fn shutdown() {
        tokio::signal::windows::ctrl_c()
            .expect("Failed to register signal handler")
            .recv()
            .await;
        info!("hbx::signal received {}, starting shutdown", name);
    }
}
