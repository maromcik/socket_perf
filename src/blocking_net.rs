use crate::config::{calculate_gb, calculate_mb, ClientConfig, ServerConfig, Stats};
use crate::error::AppError;
use log::{error, info};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Barrier, mpsc};
use std::thread;

pub fn run_blocking_server(config: &ServerConfig) -> Result<(), AppError> {
    let addr = format!("{}:{}", config.ip, config.port);
    let listener = TcpListener::bind(addr.as_str())?;
    info!("(blocking) Server listening on {}", addr);

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
            return Ok(());
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

pub fn run_threaded_blocking_clients(config: &ClientConfig) -> Result<(), AppError> {
    let barrier = Arc::new(Barrier::new(config.threads));
    let (tx, rx) = mpsc::channel();
    for _ in 0..config.threads {
        let tx = tx.clone();
        let br = barrier.clone();
        let c = config.clone();
        thread::spawn(move || {
            let res = run_blocking_client(c, br);
            if let Err(e) = tx.send(res) {
                error!("Error sending result: {e:?}");
            }
        });
    }
    drop(tx);
    let mut grand_total_bytes: u64 = 0;
    let mut grand_total_packets: u64 = 0;
    for received in rx {
        match received {
            Ok(stat) => {
                grand_total_bytes += stat.total_bytes;
                grand_total_packets += stat.total_packets;
                let mbps = calculate_mb(stat.total_bytes) / config.duration.as_secs_f64();
                info!(
                    "Stream speed: {:.2} Mbps; Stream packets: {:.2}",
                    mbps,
                    stat.total_packets as f64 / config.duration.as_secs_f64()
                );
            }
            Err(e) => {
                error!("Thread Error: {e:?}");
            }
        }
    }

    let gbps = calculate_gb(grand_total_bytes) / config.duration.as_secs_f64();
    info!("Total speed in all streams: {:.3} Gbps", gbps);
    info!(
        "Total packet count in all streams: {:.2}",
        grand_total_packets as f64 / config.duration.as_secs_f64()
    );
    Ok(())
}

pub fn run_blocking_client(config: ClientConfig, barrier: Arc<Barrier>) -> Result<Stats, AppError> {
    let addr = format!("{}:{}", config.ip, config.port);
    let stream = TcpStream::connect(addr.as_str())?;
    stream.set_nodelay(true)?;
    info!("(blocking) Connected to {addr}");

    let mut writer: Box<dyn Write + Unpin + Send> = if config.buffer_size > 0 {
        Box::new(std::io::BufWriter::with_capacity(
            config.buffer_size,
            stream,
        ))
    } else {
        Box::new(stream)
    };

    let mut packet = vec![0u8; config.packet_size];

    let mut sent_bytes: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_packets: u64 = 0;
    let mut packet_count = 0_u64;
    let mut i = 0_u128;
    barrier.wait();
    let total_duration = std::time::Instant::now();
    barrier.wait();
    let mut last = std::time::Instant::now();
    while total_duration.elapsed() <= config.duration {
        if last.elapsed().as_secs_f64() >= 1.0 {
            let mbps = calculate_mb(sent_bytes);
            info!("(blocking) Sent {:.2} Mbps; {packet_count} packets", mbps);
            sent_bytes = 0;
            packet_count = 0;
            last = std::time::Instant::now();

        }

        if config.changing_data {
            packet.extend(i.to_string().as_bytes());
            i += 1;
        }
        writer.write_all(&packet)?;
        total_bytes += config.packet_size as u64;
        sent_bytes += config.packet_size as u64;
        total_packets += 1;
        packet_count += 1;
        if config.buffer_size > 0 && sent_bytes.is_multiple_of(config.buffer_size as u64) {
            writer.flush()?;
        }

        if config.changing_data {
            packet = vec![0u8; config.packet_size];
        }
    }
    Ok(Stats {
        total_bytes,
        total_packets,
    })
}

