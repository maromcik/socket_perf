use crate::async_net::{run_async_clients, run_async_server};
use crate::blocking_net::{run_blocking_server, run_threaded_blocking_clients};
use crate::config::{Args, ClientConfig, ServerConfig};
use crate::error::AppError;
use clap::Parser;
use log::warn;
use std::time::Duration;

mod async_net;
mod blocking_net;
mod config;
mod error;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Args::parse();

    env_logger::Builder::new()
        .filter(None, args.log_level)
        .init();

    match (args.bind, args.connect) {
        (None, Some(connect)) => {
            let config = ClientConfig {
                ip: connect,
                port: args.port,
                threads: args.threads,
                packet_size: args.size,
                buffer_size: args.buffer,
                changing_data: args.variable_data,
                duration: Duration::from_secs(args.duration),
            };
            if args.use_async {
                run_async_clients(&config).await?;
            } else {
                run_threaded_blocking_clients(&config)?;
            }
        }
        (Some(bind), None) => {
            let config = ServerConfig {
                ip: bind,
                port: args.port,
            };
            if args.use_async {
                run_async_server(&config).await?
            } else {
                run_blocking_server(&config)?;
            }
        }
        (_, _) => warn!("Must specify either --bind (-b) or --connect (-c)"),
    }

    Ok(())
}
