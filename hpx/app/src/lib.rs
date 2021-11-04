use structopt::StructOpt;

mod config;

pub use config::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "HPX")]
pub struct App {
    /// Activate debug mode
    #[structopt(short = "l", long = "level", default_value = "INFO")]
    pub level: String,
    /// runtime work thread
    #[structopt(short = "w", long = "worker", default_value = "6")]
    pub worker_thread: usize,
    /// server bind port
    #[structopt(short = "p", long = "port", default_value = "80")]
    pub port: u16,
}
