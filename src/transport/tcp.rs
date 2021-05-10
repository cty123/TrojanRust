use super::super::protocol;

pub async fn dispatch(socket: tokio::net::TcpStream) -> Result<(), String> {
    // TODO: Should decide which protocol to use based on configs
    let mut handler = protocol::socks5::handler::Handler::new(socket);

    return match handler.handle().await {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }

    // let mut buf = [0; 1024];

    // let n = match socket.read(&mut buf).await {
    //     Ok(n) => n,
    //     Err(e) => {
    //         warn!("failed to read from socket; err = {:?}", e);
    //         return Err("Failed to read socket");
    //     }
    // };

    // // debug!("Read {} bytes of data\n {:?}", n, vec!(&buf[0..50]));

    // // Socks5 ack
    // let res = [5, 0];
    // if let Err(e) = socket.write_all(&res).await {
    //     error!("failed to write to socket; err = {:?}", e);
    // }

    // // Read request
    // match socket.read(&mut buf).await {
    //     Ok(n) => n,
    //     Err(e) => {
    //         warn!("failed to read from socket; err = {:?}", e);
    //         return Err("Failed to read socket");
    //     }
    // };

    // let request = match protocol::socks5::parser::parse(&buf) {
    //     Ok(r) => r,
    //     Err(e) => {
    //         warn!("Error parsing socks5 request datagram: {}", e);
    //         return Err("Error parsing socks5 request");
    //     }
    // };

    // let reply = [5, 0, 0, 1, 127, 0, 0, 1, 0x1f, 0x90];
    // if let Err(e) = socket.write_all(&reply).await {
    //     error!("failed to write to socket; err = {:?}", e);
    // }

    // // Establish remote connection to socks target
    // let target = tokio::net::TcpStream::connect(request.request_addr_port());

    // debug!("Dialing TCP connection to {}", request.request_addr_port());

    // loop {
    //     let n = match socket.read(&mut buf).await {
    //         Ok(n) if n == 0 => {
    //             info!("socket closed");
    //             break;
    //         }
    //         Ok(n) => n,
    //         Err(e) => {
    //             warn!("failed to read from socket; err = {:?}", e);
    //             return Err("Failed to read socket");
    //         }
    //     };

        

    //     debug!("Read {} bytes of data", n);
    // }

    // Ok(())
}
