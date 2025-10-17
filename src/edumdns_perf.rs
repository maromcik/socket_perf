// let im = IpMetadata {
//     src_ip: IpNetwork::default_ipv4(),
//     dst_ip: IpNetwork::default_ipv4(),
// };
// let pm = PacketMetadata {
//     datalink_metadata: DataLinkMetadata::default(),
//     ip_metadata: im,
//     transport_metadata: Default::default(),
// };
// let meta = ProbeMetadata {
//     id: Uuid("01998796bc8e7ce4bda8cafdb4f72c02".parse::<uuid::Uuid>()?),
//     mac: MacAddr::default(),
//     ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
// };
// let payload = vec![0u8; 80];
// let pp = ProbePacket {
//     probe_metadata: meta,
//     packet_metadata: pm,
//     payload,
//     payload_hash: "".to_string(),
// };
// let p = NetworkAppPacket::Data(pp);
//
// let h = handle.clone();
// tokio::task::spawn(async move {
//     let mut t = Instant::now();
//     let interval = Duration::from_secs(1);
//     let mut packet_count: i64 = 0;
//     loop {
//         let _ = h
//             .send_message_with_response(|tx| {
//                 TcpConnectionMessage::send_packet(tx, p.clone(), false)
//             })
//             .await;
//         packet_count += 1;
//             if t.elapsed() > interval {
//                 info!("Sent {} packets", packet_count);
//                 t = Instant::now();
//                 packet_count = 0;
//             }
//     }
// });