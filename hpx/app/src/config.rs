use std::env;

#[derive(Debug)]
pub struct Config {
    pub tracing_udp: Option<String>,
    pub sampling_percentage: usize,
    pub env_code: String,
    pub connect_timeout: usize,
    pub keepalive_timeout: usize,
}

const DEFAULT_SAMPLING_PERCENTAGE: usize = 0;
const DEFAULT_CONNECT_TIMEOUT: usize = 30;
const DEFAULT_KEEPALIVE_TIMEOUT: usize = 60;

impl Config {
    pub fn init() -> Self {
        let udp = env::var("OPEN_TRACING").ok();
        let percentage = parse_env_num("SAMPLING_PERCENTAGE", DEFAULT_SAMPLING_PERCENTAGE);
        let connect_timeout = parse_env_num("CONNECT_TIMEOUT", DEFAULT_CONNECT_TIMEOUT);
        let keepalive_timeout = parse_env_num("KEEPALIVE_TIMEOUT", DEFAULT_KEEPALIVE_TIMEOUT);
        let env_code = env::var("ENV_CODE").expect("ENV_CODE is empty!");
        Self {
            tracing_udp: udp,
            sampling_percentage: percentage,
            env_code,
            connect_timeout,
            keepalive_timeout,
        }
    }
}

fn parse_env_num(key: &str, default: usize) -> usize {
    env::var(key).map_or(default, |v| v.parse().unwrap_or(default))
}
