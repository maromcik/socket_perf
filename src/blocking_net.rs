use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub fn run_blocking_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr)?;
    println!("(blocking) Server listening on {addr}");

    let (mut stream, peer) = listener.accept()?;
    println!("(blocking) Client connected: {peer}");

    let mut buf = vec![0u8; 1024 * 1024];
    let mut total_bytes: u64 = 0;
    let mut last = std::time::Instant::now();

    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            println!("(blocking) Connection closed");
            break;
        }
        total_bytes += n as u64;

        if last.elapsed().as_secs_f64() >= 1.0 {
            let mbps = (total_bytes as f64 * 8.0) / 1_000_000.0;
            println!("(blocking) Received {:.2} Mbps", mbps);
            total_bytes = 0;
            last = std::time::Instant::now();
        }
    }
    Ok(())
}

pub fn run_blocking_client(addr: &str, packet_size: usize) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(addr)?;
    stream.set_nodelay(true)?;
    println!("(blocking) Connected to {addr}");
    
    let packet = vec![0u8; packet_size];

    let mut sent_bytes: u64 = 0;
    let mut last = std::time::Instant::now();
    let mut packet_count = 0_u64;
    loop {
        stream.write_all(&packet)?;
        sent_bytes += packet_size as u64;
        packet_count += 1;
        if last.elapsed().as_secs_f64() >= 1.0 {
            let mbps = (sent_bytes as f64 * 8.0) / 1_000_000.0;
            println!("(blocking) Sent {:.2} Mbps", mbps);
            println!("(blocking) Sent {} packets", packet_count);
            sent_bytes = 0;
            last = std::time::Instant::now();
            packet_count = 0;
        }
    }
}