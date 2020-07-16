use tokio::net::TcpStream;

trait Packet {
    fn get_id() -> u8;
    fn parse(stream: TcpStream) -> Self;
}