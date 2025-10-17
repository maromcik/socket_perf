use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWrite, BufWriter};
use std::error::Error;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

pub async fn run_async_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("(async) Server listening on {addr}");

    loop {

        let (socket, peer) = listener.accept().await?;
        println!("Client connected: {peer:?}");
        tokio::spawn(
            async move {
                handle_connection(socket).await }
        );
    }
}

pub async fn handle_connection(mut socket: TcpStream) {
    let mut buf = vec![0u8; 1024 * 1024]; // 64 KB read buffer
    let mut total_bytes: u64 = 0;
    let mut last = tokio::time::Instant::now();
    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed by peer");
                break;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Read error: {e:?}");
                break;
            }
        };

        total_bytes += n as u64;

        // print throughput every second
        if last.elapsed().as_secs_f64() >= 1.0 {
            let mbps = (total_bytes as f64 * 8.0) / 1_000_000.0;
            println!("Received {:.2} Mbps", mbps);
            total_bytes = 0;
            last = tokio::time::Instant::now();
        }
    }
}



pub async fn run_async_client(
    addr: &str,
    packet_size: usize,
    buffer_size: usize,
) -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect(addr).await?;
    stream.set_nodelay(true)?;
    println!("Connected to server {addr}");
    // Wrap in BufWriter if batching is enabled
    let mut writer: Box<dyn AsyncWrite + Unpin + Send> = if buffer_size > 0 {
        Box::new(BufWriter::with_capacity(buffer_size, stream))
    } else {
        Box::new(stream)
    };

    let packet = vec![0u8; packet_size];
    let mut sent_bytes: u64 = 0;
    let mut last = tokio::time::Instant::now();
    let mut packet_count = 0_u64;
    loop {
        writer.write_all(&packet).await?;
        sent_bytes += packet_size as u64;
        packet_count += 1;
        // Only flush if we're using batching
        if buffer_size > 0 && sent_bytes % (buffer_size as u64) == 0 {
            writer.flush().await?;
        }

        if last.elapsed().as_secs_f64() >= 1.0 {
            let mbps = (sent_bytes as f64 * 8.0) / 1_000_000.0;
            println!("(async) Sent {:.2} Mbps", mbps);
            println!("(async) Sent {} packets", packet_count);
            sent_bytes = 0;
            last = tokio::time::Instant::now();
            packet_count = 0;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}