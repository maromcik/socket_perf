//
// let im = IpMetadata { src_ip: IpNetwork::default_ipv4(), dst_ip: IpNetwork::default_ipv4(), };
// let pm = PacketMetadata { datalink_metadata: DataLinkMetadata::default(), ip_metadata: im, transport_metadata: Default::default(), };
// let meta = ProbeMetadata {
// id: Uuid("01998796bc8e7ce4bda8cafdb4f72c02".parse::<uuid::Uuid>()?),
// mac: MacAddr::default(),
// ip: IpAddr::V4(Ipv4::new(127, 0, 0, 1)),
// };
// let payload = vec![0u8; 80];
// let pp = ProbePacket {
// probe_metadata: meta,
// packet_metadata: pm,
// payload,
// payload_hash: "kokot".to_string(),
// };
// let p = NetworkAppPacket::Data(pp);
//
// let h = handle.clone();
// tokio::task::spawn(async move {
// loop {
// let _ = h
// .send_message_with_response(|tx| {
// TcpConnectionMessage::send_packet(tx, p.clone())
// })
// .await;
// }
// });