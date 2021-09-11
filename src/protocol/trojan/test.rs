// use std::io::Result;

// /// Tests for PacketTrojanOutboundStream
// pub struct UdpSocket {}
// impl UdpSocket {
//     pub async fn bind(_s: &str) -> Result<UdpSocket> {
//         print!("Mocking UdpSocket");
//         Ok(UdpSocket {})
//     }

//     pub fn poll_recv_from(
//         &self,
//         _cx: &mut std::task::Context<'_>,
//         _buf: &mut tokio::io::ReadBuf<'_>,
//     ) -> std::task::Poll<std::io::Result<std::net::SocketAddr>> {
//         std::task::Poll::Ready(Ok(std::net::SocketAddr::from(([127, 0, 0, 1], 80))))
//     }

//     pub fn poll_send_to(
//         &self,
//         _cx: &mut std::task::Context<'_>,
//         _buf: &[u8],
//         _target: std::net::SocketAddr,
//     ) -> std::task::Poll<std::io::Result<usize>> {
//         std::task::Poll::Ready(Ok(10usize))
//     }}

// #[tokio::test]
// async fn test_function() {
//     println!("Start");
//     let mut stream = crate::protocol::trojan::packet::PacketTrojanOutboundStream::new()
//         .await
//         .unwrap();
//     use tokio::io::AsyncWriteExt;
//     let mut buf = [1, 2, 3];
//     stream.write(&buf);
//     assert_eq!(1, 1);
//     println!("Hello");
// }
