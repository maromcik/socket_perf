use clap::{Parser, ValueEnum};
use std::error::Error;
use std::io::{ Write};
use crate::async_net::{run_async_client, run_async_server};
use crate::blocking_net::{run_blocking_client, run_blocking_server};

mod blocking_net;
mod async_net;

#[derive(Parser, Debug)]
#[command(name = "tcp_blast_dual", about = "Compare async vs sync TCP throughput")]
struct Args {

    /// Address to bind or connect to
    #[arg(short, long, default_value = "127.0.0.1:3003")]
    addr: String,

    #[arg(short = 'l', long, default_value = "1500")]
    length: usize,

    #[arg(short, long, default_value = "0")]
    buffer: usize,

    #[arg(short = 's', long = "server", action = clap::ArgAction::SetTrue)]
    server: bool,

    #[arg(short = 'c', long = "client", action = clap::ArgAction::SetTrue)]
    client: bool,

    #[arg(short = 'd', long = "changing_data", action = clap::ArgAction::SetTrue)]
    changing_data: bool,

    #[arg(short = 'a', long = "async", action = clap::ArgAction::SetTrue)]
    use_async: bool,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.use_async {
        match (args.server, args.client) {
            (false, true) => run_async_client(&args.addr, args.size, args.buffer, args.changing_data).await?,
            (true, false) => run_async_server(&args.addr).await?,
            _ => panic!("Must specify either --server or --client"),
        }
    } else {
        match (args.server, args.client) {
          (false, true)   => run_blocking_client(&args.addr ,args.size, args.buffer, args.changing_data)?,
          (true, false)   => run_blocking_server(&args.addr)?,
            _ => panic!("Must specify either --server or --client"),
        }
    }

    Ok(())
}