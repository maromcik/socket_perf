use clap::{Parser, ValueEnum};
use std::error::Error;
use std::io::{Read, Write};
use crate::async_net::{run_async_client, run_async_server};
use crate::blocking_net::{run_blocking_client, run_blocking_server};

mod blocking_net;
mod async_net;
mod edumdns_perf;

#[derive(Parser, Debug)]
#[command(name = "tcp_blast_dual", about = "Compare async vs sync TCP throughput")]
struct Args {
    /// Mode: server or client
    #[arg(short, long, value_enum)]
    mode: Mode,

    /// Address to bind or connect to
    #[arg(short, long, default_value = "127.0.0.1:3003")]
    addr: String,

    /// Use Tokio async runtime
    #[arg(long, default_value_t = false)]
    r#async: bool,

    #[arg(short, long, default_value = "1500")]
    size: usize,

    #[arg(short, long, default_value = "0")]
    buffer: usize,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Mode {
    Server,
    Client,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.r#async {
        match args.mode {
            Mode::Client => run_async_client(&args.addr, args.size, args.buffer).await?,
            Mode::Server => run_async_server(&args.addr).await?,
        }
    } else {
        match args.mode {
            Mode::Client => run_blocking_client(&args.addr ,args.size, args.buffer)?,
            Mode::Server => run_blocking_server(&args.addr)?,
        }
    }

    Ok(())
}