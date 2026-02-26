use std::sync::{Arc};
use crate::error::AppError;
use log::{error, info};
use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncReadExt, AsyncWrite, BufWriter};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::task;
use crate::config::{calculate_gb, calculate_mb, ClientConfig, ServerConfig, Stats};

pub async fn run_async_servers(config: &ServerConfig) -> Result<(), AppError> {
    let mut tasks = Vec::new();
    for i in (config.port as usize)..config.port as usize + config.threads {
        let addr = format!("{}:{i}", config.ip);
        tasks.push(task::spawn(async move {
            if let Err(e) = run_async_server(addr.as_str()).await {
                error!("Error for {addr}: {e:?}");
            }
        }));
    }
    for t in tasks {
        if let Err(e) = t.await {
            error!("Thread panicked: {e:?}");
        }
    }
    Ok(())
}


pub async fn run_async_server(addr: &str) -> Result<(), AppError> {
    let listener = TcpListener::bind(addr).await?;
    info!("(async) Server listening on {addr}");

    loop {
        let (socket, peer) = listener.accept().await?;
        info!("Client connected: {peer:?}");
        tokio::spawn(async move { handle_connection(socket).await });
    }
}

pub async fn handle_connection(mut socket: TcpStream) {
    let mut buf = vec![0u8; 1024 * 1024]; // 64 KB read buffer
    let mut total_bytes: u64 = 0;
    let mut last = tokio::time::Instant::now();
    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) => {
                info!("Connection closed by peer");
                break;
            }
            Ok(n) => n,
            Err(e) => {
                error!("Read error: {e:?}");
                break;
            }
        };

        total_bytes += n as u64;

        // print throughput every second
        if last.elapsed().as_secs_f64() >= 1.0 {
            let mbps = (total_bytes as f64 * 8.0) / 1_000_000.0;
            info!("Received {:.2} Mbps", mbps);
            total_bytes = 0;
            last = tokio::time::Instant::now();
        }
    }
}

pub async fn run_async_clients(config: &ClientConfig) -> Result<(), AppError> {
    let barrier = Arc::new(tokio::sync::Barrier::new(config.threads));
    let (tx, mut rx) = tokio::sync::mpsc::channel(config.threads*10);
    for i in (config.port as usize)..config.port as usize + config.threads {
        let tx = tx.clone();
        let br = barrier.clone();
        let mut c = config.clone();
        c.set_port(i as u16);
        task::spawn(async move {
            let res = run_async_client(c, br).await;
            if let Err(e) = tx.send(res).await {
                error!("Error sending result: {e:?}");
            }
        });
    }
    drop(tx);
    let mut grand_total_bytes: u64 = 0;
    let mut grand_total_packets: u64 = 0;
    while let Some(received) = rx.recv().await {
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

pub async fn run_async_client(
    config: ClientConfig,
    barrier: Arc<tokio::sync::Barrier>
) -> Result<Stats, AppError> {
    let addr = format!("{}:{}", config.ip, config.port);
    let stream = TcpStream::connect(addr.as_str()).await?;
    stream.set_nodelay(true)?;
    info!("Connected to server {addr}");

    let mut writer: Box<dyn AsyncWrite + Unpin + Send> = if config.buffer_size > 0 {
        Box::new(BufWriter::with_capacity(config.buffer_size, stream))
    } else {
        Box::new(stream)
    };

    let mut packet = vec![0u8; config.packet_size];
    let mut sent_bytes: u64 = 0;
    let mut packet_count = 0_u64;
    let mut total_bytes: u64 = 0;
    let mut total_packets: u64 = 0;
    let mut i = 0_u128;
    barrier.wait().await;
    let total_duration = tokio::time::Instant::now();
    barrier.wait().await;
    let mut last = tokio::time::Instant::now();
    while total_duration.elapsed() <= config.duration {
        if config.changing_data {
            packet.extend(i.to_string().as_bytes());
            i += 1;
        }
        writer.write_all(&packet).await?;
        total_bytes += config.packet_size as u64;
        sent_bytes += config.packet_size as u64;
        packet_count += 1;
        total_packets += 1;

        if config.buffer_size > 0 && sent_bytes.is_multiple_of(config.buffer_size as u64) {
            writer.flush().await?;
        }

        if last.elapsed().as_secs_f64() >= 1.0 {
            let mbps = calculate_mb(sent_bytes);
            info!("(async) Sent {:.2} Mbps; {packet_count} packets", mbps);
            sent_bytes = 0;
            last = tokio::time::Instant::now();
            packet_count = 0;
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
