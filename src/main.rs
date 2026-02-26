use std::sync::Arc;
use crate::async_net::{run_async_client, run_async_server};
use crate::blocking_net::{run_n_clients, run_n_servers};
use clap::{Parser};
use std::time::Duration;
use log::warn;
use crate::error::AppError;

mod async_net;
mod blocking_net;
mod error;

#[derive(Parser, Debug)]
#[command(
    name = "tcp_blast_dual",
    about = "Compare async vs sync TCP throughput"
)]
struct Args {
    #[arg(short = 's', long, default_value = "1500")]
    size: usize,

    #[arg(short = 'f', long = "buffer", default_value = "0")]
    buffer: usize,

    #[arg(short = 'p', long = "port", default_value = "3003")]
    port: u16,

    #[arg(
        short = 'l',
        long,
        default_value = "info",
        env = "RUST_LOG",
        value_name = "LOG_LEVEL"
    )]
    log_level: log::LevelFilter,

    #[arg(short = 'b', long = "bind")]
    bind: Option<String>,

    #[arg(short = 'c', long = "connect")]
    connect: Option<String>,

    #[arg(short = 'v', long = "changing_data", action = clap::ArgAction::SetTrue)]
    variable_data: bool,

    #[arg(short = 'a', long = "async", action = clap::ArgAction::SetTrue)]
    use_async: bool,

    #[arg(short = 't', long = "threads", default_value = "1")]
    threads: usize,

    #[arg(short = 'd', long = "duration", default_value = "10")]
    duration: u64,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Args::parse();

    env_logger::Builder::new()
        .filter(None, args.log_level)
        .init();

    match (args.bind, args.connect) {
        (None, Some(connect)) => {
            if args.use_async {
                run_async_client(format!("{connect}:{}", args.port).as_str(), args.size, args.buffer, args.variable_data).await?;
            } else {
                
                run_n_clients(connect.as_str(), args.port as usize, args.size, args.buffer, args.variable_data, Duration::from_secs(args.duration), args.threads)?;
            }
        }
        (Some(bind), None) => {
            if args.use_async {
                run_async_server(format!("{bind}:{}", args.port).as_str()).await?
            } else {
                run_n_servers(bind.as_str(), args.port as usize, args.threads)?;
            }
        }
        (_, _) => warn!("Must specify either --bind (-b) or --connect (-c)"),
    }

    Ok(())
}
