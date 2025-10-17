use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn run_blocking_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr)?;
    println!("(blocking) Server listening on {addr}");


    loop {
        let (stream, peer) = listener.accept()?;
        println!("Client connected: {peer}");
        thread::spawn(move || {
            let _ = handle_connection(stream);    
        });
        
    }

}
pub fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = vec![0u8; 1024 * 1024];
    let mut total_bytes: u64 = 0;
    let mut last = std::time::Instant::now();

    loop {
        let n = socket.read(&mut buf)?;
        if n == 0 {
            println!("Connection closed");
            return Ok(())
        }
        total_bytes += n as u64;

        if last.elapsed().as_secs_f64() >= 1.0 {
            let mbps = (total_bytes as f64 * 8.0) / 1_000_000.0;
            println!("Received {:.2} Mbps", mbps);
            total_bytes = 0;
            last = std::time::Instant::now();
        }
    }
}

pub fn run_blocking_client(addr: &str, packet_size: usize, buffer_size: usize) -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect(addr)?;
    stream.set_nodelay(true)?;
    println!("(blocking) Connected to {addr}");
    
    let mut writer: Box<dyn Write + Unpin + Send> = if buffer_size > 0 {
        Box::new(std::io::BufWriter::with_capacity(buffer_size, stream))
    } else {
        Box::new(stream)
    };
    
    let packet = vec![0u8; packet_size];

    let mut sent_bytes: u64 = 0;
    let mut last = std::time::Instant::now();
    let mut packet_count = 0_u64;
    loop {
        writer.write_all(&packet)?;
        sent_bytes += packet_size as u64;
        packet_count += 1;
        if buffer_size > 0 && sent_bytes % (buffer_size as u64) == 0 {
            writer.flush()?;
        }
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