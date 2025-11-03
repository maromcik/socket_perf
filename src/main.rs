use crate::async_net::{run_async_client, run_async_server};
use crate::blocking_net::{run_blocking_client, run_blocking_server};
use clap::{Parser, arg};
use std::error::Error;
use log::warn;

mod async_net;
mod blocking_net;

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

    #[arg(
        short = 'l',
        long,
        default_value = "info",
        env = "RUST_LOG",
        value_name = "LOG_LEVEL"
    )]
    log_level: log::LevelFilter,

    #[arg(short = 'b', long = "bind", action = clap::ArgAction::SetTrue)]
    bind: Option<String>,

    #[arg(short = 'c', long = "connect", action = clap::ArgAction::SetTrue)]
    connect: Option<String>,

    #[arg(short = 'd', long = "changing_data", action = clap::ArgAction::SetTrue)]
    changing_data: bool,

    #[arg(short = 'a', long = "async", action = clap::ArgAction::SetTrue)]
    use_async: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    env_logger::Builder::new()
        .filter(None, args.log_level)
        .init();

    match (args.bind, args.connect) {
        (None, Some(connect)) => {
            if args.use_async {
                run_async_client(&connect, args.size, args.buffer, args.changing_data).await?;
            } else {
                run_blocking_client(&connect, args.size, args.buffer, args.changing_data)?;
            }
        }
        (Some(bind), None) => {
            if args.use_async {
                run_async_server(&bind).await?
            } else {
                run_blocking_server(&bind)?;
            }
        }
        (_, _) => warn!("Must specify either --server or --client"),
    }

    Ok(())
}
