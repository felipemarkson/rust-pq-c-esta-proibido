use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::UdpSocket;

const DATA_LIMIT: usize = 4096;
const PORT: u16 = 9999;
const PORT_BACKEND: u16 = 8888;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], PORT)))?;
    let conn2backend = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], PORT - 1)))?;
    conn2backend.connect(SocketAddr::from(([127, 0, 0, 1], PORT_BACKEND)))?;

    for conn in listener.incoming() {
        println!("Server: I recived a connection!");
        if let Err(e) = &conn {
            eprintln!("Server: Could not open connection: {}", e);
            continue;
        }
        let mut conn = conn.unwrap();
        let mut buf = [0_u8; DATA_LIMIT];
        if let Err(e) = conn.read(&mut buf) {
            eprintln!("Server: Could not read: {}", e);
            continue;
        }

        if let Err(e) = conn2backend.send(&buf) {
            eprintln!("Could not send to backend: {}", e);
            let _ = conn.write(b"HTTP/1.1 500 Internal Error\r\nReason: Send2Back\r\n\r\n");
            continue;
        };

        let nbytes = conn2backend.recv(&mut buf);
        if let Err(e) = nbytes {
            eprintln!("Could not recv from backend: {}", e);
            let _ = conn.write(b"HTTP/1.1 500 Internal Error\r\nReason: RecvFromBack\r\n\r\n");
            continue;
        }
        let buf = &buf[..nbytes.unwrap()];
        let _ = conn.write(buf);
    }

    Ok(())
}
