use clap::Parser;
use std::net::IpAddr;

#[derive(Parser, Debug)]
#[command(name = "socket_perf", about = "Measure TCP throughput")]
pub struct Args {
    #[arg(short = 's', long, default_value = "1500")]
    pub size: usize,

    #[arg(short = 'f', long = "buffer", default_value = "0")]
    pub buffer: usize,

    #[arg(short = 'p', long = "port", default_value = "3003")]
    pub port: u16,

    #[arg(
        short = 'l',
        long,
        default_value = "info",
        env = "RUST_LOG",
        value_name = "LOG_LEVEL"
    )]
    pub log_level: log::LevelFilter,

    #[arg(short = 'b', long = "bind")]
    pub bind: Option<IpAddr>,

    #[arg(short = 'c', long = "connect")]
    pub connect: Option<IpAddr>,

    #[arg(short = 'v', long = "changing_data", action = clap::ArgAction::SetTrue)]
    pub variable_data: bool,

    #[arg(short = 'a', long = "async", action = clap::ArgAction::SetTrue)]
    pub use_async: bool,

    #[arg(short = 't', long = "threads", default_value = "1")]
    pub threads: usize,

    #[arg(short = 'd', long = "duration", default_value = "10")]
    pub duration: u64,
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub total_bytes: u64,
    pub total_packets: u64,
}

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub ip: IpAddr,
    pub port: u16,
    pub threads: usize,
    pub packet_size: usize,
    pub buffer_size: usize,
    pub changing_data: bool,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub ip: IpAddr,
    pub port: u16,
}

pub fn calculate_mb(val: u64) -> f64 {
    (val as f64 * 8.0) / 1_000_000.0
}

pub fn calculate_gb(val: u64) -> f64 {
    (val as f64 * 8.0) / 1_000_000_000.0
}
