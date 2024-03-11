use std::net::{SocketAddr, UdpSocket};

mod client;
use client::Client;
use database::{
    BufferExtrato, BufferOperation, BufferTranscaoReturn, Converter, Operation, OperationKind,
    TransacaoReturn, PORT_DB, RES_ERROR, SIZE_OPERATION,
};

fn send_buffer(socket: &UdpSocket, buffer: &[u8], addr: &SocketAddr) {
    if let Err(e) = socket.send_to(buffer, addr) {
        eprint!("DB: Could not respond to {}: {}", addr, e);
    }
}

fn main() -> std::io::Result<()> {
    let clients = &mut [
        Client::load_client(1).unwrap_or_else(|| Client::new(1, 100_000, 0)),
        Client::load_client(2).unwrap_or_else(|| Client::new(2, 80_000, 0)),
        Client::load_client(3).unwrap_or_else(|| Client::new(3, 1_000_000, 0)),
        Client::load_client(4).unwrap_or_else(|| Client::new(4, 10_000_000, 0)),
        Client::load_client(5).unwrap_or_else(|| Client::new(5, 500_000, 0)),
    ];
    for client in clients.iter() {
        println!("{:?}", client);
    }

    let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], PORT_DB)))?;

    loop {
        let mut buf: BufferOperation = [0; SIZE_OPERATION];
        let (size, addr) = socket.recv_from(&mut buf)?;
        if size != SIZE_OPERATION {
            eprint!("DB: Invalid object recived. Size = {}", size);
            if socket.send_to(&RES_ERROR, addr).is_err() {
                eprint!("DB: Could not respond to {}", addr);
            }
        }
        let op: Operation = Converter::from_buffer(&buf);

        match op.kind {
            OperationKind::Extrato => {
                let client = clients.get(op.id as usize - 1);
                if client.is_none() {
                    eprint!("DB: Invalid id {}", op.id);
                    send_buffer(&socket, &RES_ERROR, &addr);
                    continue;
                }

                let client = client.unwrap();
                let buf: BufferExtrato = Converter::to_buffer(&client.extrato());
                send_buffer(&socket, &buf, &addr);
            }
            OperationKind::Transacao => {
                let client = clients.get_mut(op.id as usize - 1);
                if client.is_none() {
                    eprint!("DB: Invalid id {}", op.id);
                    send_buffer(&socket, &RES_ERROR, &addr);
                    continue;
                }
                let client = client.unwrap();
                if client.push_transacao(op.transacao).is_err() {
                    send_buffer(&socket, &RES_ERROR, &addr);
                    continue;
                }
                let ret = TransacaoReturn {
                    limite: client.limite,
                    saldo: client.saldo,
                };
                let buf: BufferTranscaoReturn = Converter::to_buffer(&ret);
                send_buffer(&socket, &buf, &addr);
            }
        }
    }
}
