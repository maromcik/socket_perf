use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use log::{error, info};
use crate::error::AppError;

pub fn run_blocking_server(addr: &str) -> Result<(), AppError> {
    let listener = TcpListener::bind(addr)?;
    info!("(blocking) Server listening on {addr}");


    loop {
        let (stream, peer) = listener.accept()?;
        info!("Client connected: {peer}");
        thread::spawn(move || {
            let _ = handle_connection(stream);    
        });
        
    }

}
pub fn handle_connection(mut socket: TcpStream) -> Result<(), AppError> {
    let mut buf = vec![0u8; 1024 * 1024];
    let mut total_bytes: u64 = 0;
    let mut last = std::time::Instant::now();

    loop {
        let n = socket.read(&mut buf)?;
        if n == 0 {
            info!("Connection closed");
            return Ok(())
        }
        total_bytes += n as u64;

        if last.elapsed().as_secs_f64() >= 1.0 {
            let mbps = (total_bytes as f64 * 8.0) / 1_000_000.0;
            info!("Received {:.2} Mbps", mbps);
            total_bytes = 0;
            last = std::time::Instant::now();
        }
    }
}

pub fn run_n_servers(ip: &str, start_port: usize, n: usize) -> Result<(), AppError> {
    let mut threads = Vec::new();
    for i in start_port..start_port+n {
        let addr = format!("{ip}:{i}");
        threads.push(thread::spawn(move || {
            if let Err(e) = run_blocking_server(addr.as_str()) {
                error!("Error for {addr}: {e:?}");
            }
        }));
    }
    for t in threads {
        if let Err(e) = t.join() {
            error!("Thread panicked: {e:?}");
        }
    };
    Ok(())
}

pub fn run_n_clients(ip: &str, start_port: usize, packet_size: usize, buffer_size: usize, changing_data: bool, duration: Duration, n: usize) -> Result<(), AppError> {
    let (tx, rx) = mpsc::channel();
    for i in start_port..start_port+n {
        let addr = format!("{ip}:{i}");
        let tx = tx.clone();
        thread::spawn(move || {
            let res = run_blocking_client(addr.as_str(), packet_size, buffer_size, changing_data, duration);
            if let Err(e) = tx.send(res) {
                error!("Error sending result: {e:?}");
            }

        });
    }
    drop(tx);
    let mut grand_total: u64 = 0;
    for received in rx {
        match received {
            Ok(total_bytes) => {
                grand_total += total_bytes;

                let mbps = calculate_mb(total_bytes) / duration.as_secs_f64();
                info!("Speed in this stream: {mbps} Mbps");
            }
            Err(e) => {
                error!("Thread Error: {e:?}");
            }
        }
    }

    let gbps = calculate_gb(grand_total) / duration.as_secs_f64();
    info!("Total speed in all streams: {gbps} Gbps");
    Ok(())
}

pub fn run_blocking_client(addr: &str, packet_size: usize, buffer_size: usize, changing_data: bool, duration: Duration) -> Result<u64, AppError> {
    let stream = TcpStream::connect(addr)?;
    stream.set_nodelay(true)?;
    info!("(blocking) Connected to {addr}");
    
    let mut writer: Box<dyn Write + Unpin + Send> = if buffer_size > 0 {
        Box::new(std::io::BufWriter::with_capacity(buffer_size, stream))
    } else {
        Box::new(stream)
    };
    
    let mut packet = vec![0u8; packet_size];

    let mut sent_bytes: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut last = std::time::Instant::now();
    let mut packet_count = 0_u64;
    let total_duration = std::time::Instant::now();
    let mut i = 0_u128;
    while total_duration.elapsed() <= duration {
        if changing_data {
            packet.extend(i.to_string().as_bytes());
            i += 1;
        }
        writer.write_all(&packet)?;
        total_bytes += packet_size as u64;
        sent_bytes += packet_size as u64;
        packet_count += 1;
        if buffer_size > 0 && sent_bytes % (buffer_size as u64) == 0 {
            writer.flush()?;
        }
        if last.elapsed().as_secs_f64() >= 1.0 {
            let mbps = calculate_mb(sent_bytes);
            info!("(blocking) Sent {:.2} Mbps", mbps);
            info!("(blocking) Sent {} packets", packet_count);
            sent_bytes = 0;
            last = std::time::Instant::now();
            packet_count = 0;
        }

        if changing_data {
            packet = vec![0u8; packet_size];
        }
    }
    Ok(total_bytes)
}

pub fn calculate_mb(val: u64) -> f64 {
    (val as f64 * 8.0) / 1_000_000.0
}

pub fn calculate_gb(val: u64) -> f64 {
    (val as f64 * 8.0) / 1_000_000_000.0
}