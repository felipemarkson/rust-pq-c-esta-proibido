use std::env;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::UdpSocket;
use std::process::exit;

const DATA_LIMIT: usize = 1024;
const PORT: u16 = 9999;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("invalid number of arguments");
        exit(1);
    }
    let port1 = args[1].parse::<u16>();
    if let Err(e) = port1 {
        eprint!("Could not parse port 1: {}", e);
        exit(1);
    }
    let port1 = port1.unwrap();

    let port2 = args[2].parse::<u16>();
    if let Err(e) = port2 {
        eprint!("Could not parse port 2: {}", e);
        exit(1);
    }
    let port2 = port2.unwrap();

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], PORT)))?;
    let conn2backend1 = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], PORT - 1)))?;
    let conn2backend2 = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], PORT - 2)))?;
    conn2backend1.connect(SocketAddr::from(([127, 0, 0, 1], port1)))?;
    conn2backend2.connect(SocketAddr::from(([127, 0, 0, 1], port2)))?;

    let mut round_robin_flag = false;

    for conn in listener.incoming() {
        // println!("Server: I recived a connection!");
        if let Err(e) = &conn {
            eprintln!("Server: Could not open connection: {}", e);
            continue;
        }
        let mut conn = conn.unwrap();
        let mut buf = [0_u8; DATA_LIMIT];
        let nbytes = conn.read(&mut buf);
        if let Err(e) = nbytes {
            eprintln!("Server: Could not read: {}", e);
            continue;
        }
        let buf = &buf[..nbytes.unwrap()];

        let mut conn2backend = &conn2backend1;
        if round_robin_flag {
            conn2backend = &conn2backend2;
        }
        round_robin_flag = !round_robin_flag;

        if let Err(e) = conn2backend.send(&buf) {
            eprintln!("Could not send to backend: {}", e);
            let _ = conn.write(b"HTTP/1.1 500 Internal Error\r\nReason: Send2Back\r\n\r\n");
            continue;
        };

        let mut buf = [0_u8; DATA_LIMIT];
        let nbytes = conn2backend.recv(&mut buf);
        if let Err(e) = nbytes {
            eprintln!("Could not recv from backend: {}", e);
            let _ = conn.write(b"HTTP/1.1 500 Internal Error\r\nReason: RecvFromBack\r\n\r\n");
            continue;
        }
        let buf = &buf[..nbytes.unwrap()];
        let buf = std::str::from_utf8(buf).unwrap().trim_matches(char::from(0));
        let _ = conn.write(buf.as_bytes());
        let _ = conn.flush();
    }

    Ok(())
}
