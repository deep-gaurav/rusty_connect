use socket2::TcpKeepalive;
use tokio::net::TcpStream;

pub fn set_tcp_timeouts(stream: TcpStream) -> anyhow::Result<TcpStream> {
    let stream: std::net::TcpStream = stream.into_std()?;
    let socket: socket2::Socket = socket2::Socket::from(stream);
    let keepalive = TcpKeepalive::new()
        .with_time(std::time::Duration::from_secs(4))
        .with_interval(std::time::Duration::from_secs(1))
        .with_retries(4);
    socket.set_tcp_keepalive(&keepalive)?;
    // socket
    let stream: std::net::TcpStream = socket.into();
    let stream: tokio::net::TcpStream = tokio::net::TcpStream::from_std(stream)?;

    Ok(stream)
}
