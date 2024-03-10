use database::{
    BufferOperation, DBconn, Extrato, Operation, OperationKind, Transacao, TransacaoReturn,
    NCHAR_DESCRIPTION, SIZE_EXTRATO, SIZE_OPERATION, SIZE_TRANSACAO_RETURN,
};
use serde::{Deserialize, Serialize};
use std::{
    net::{SocketAddr, UdpSocket},
    time::{SystemTime, UNIX_EPOCH},
};

const DATA_LIMIT: usize = 4096;
const PORT: u16 = 8888;

#[derive(Serialize, Deserialize, Debug)]
struct TrasacaoBackend {
    valor: u32,
    tipo: String,
    descricao: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExtratoBackend {
    total: i64,
    limite: i64,
    transacoes: Vec<TrasacaoBackend>,
}

impl From<Extrato> for ExtratoBackend {
    fn from(extrato: Extrato) -> Self {
        let mut transacoes = Vec::new();
        for transacao in extrato.transacoes.iter() {
            if !transacao.isvalid {
                continue;
            }
            let mut tipo = String::new();
            if transacao.value < 0 {
                tipo.push('d');
            } else {
                tipo.push('c');
            }
            let mut descricao = String::new();
            for ch in transacao.transacao_description {
                if ch == '\0' {
                    break;
                }
                descricao.push(ch);
            }
            transacoes.push(TrasacaoBackend {
                valor: transacao.value as u32,
                tipo,
                descricao,
            });
        }
        ExtratoBackend {
            limite: extrato.limite,
            total: extrato.total,
            transacoes,
        }
    }
}

#[derive(PartialEq)]
enum Method {
    Get,
    Post,
}

enum Paths {
    Transacao(u8, TrasacaoBackend),
    Extrato(u8),
}

fn send_buffer(socket: &UdpSocket, buffer: &[u8], addr: &SocketAddr) {
    if let Err(e) = socket.send_to(buffer, addr) {
        eprint!("Backend: Could not respond to {}: {}", addr, e);
    }
}

#[derive(Debug)]
struct Response {
    code: usize,
    msg: &'static str,
    reason: Option<&'static str>,
    body: Option<Vec<u8>>,
}
impl Response {
    fn new(
        code: usize,
        msg: &'static str,
        reason: Option<&'static str>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Response {
            code,
            msg,
            reason,
            body,
        }
    }
    fn into_vec(self) -> Vec<u8> {
        let mut header = match self.reason {
            Some(reason) => format!(
                "HTTP/1.1 {} {}\r\nReason: {}\r\n",
                self.code, self.msg, reason
            ),
            None => format!("HTTP/1.1 {} {}\r\n", self.code, self.msg),
        };
        if self.body.is_none() {
            header.push_str("\r\n");
            return header.into();
        }
        let body = self.body.unwrap();

        header.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));

        let mut out: Vec<u8> = Vec::with_capacity(header.as_bytes().len() + body.len());
        out.extend(header.as_bytes());
        out.extend(body);
        out
    }
}

fn req_parser(buffer: &[u8]) -> Result<Paths, Response> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    let result = req.parse(buffer);
    if result.is_err() {
        return Err(Response::new(
            400,
            "Bad Request",
            Some("Invalid HTTP"),
            None,
        ));
    }

    let nbytes = result.unwrap();
    if nbytes.is_partial() {
        return Err(Response::new(
            400,
            "Bad Request",
            Some("Partial Request"),
            None,
        ));
    }
    let nbytes = nbytes.unwrap();
    if req.path.is_none() {
        return Err(Response::new(
            400,
            "Bad Request",
            Some("Unspecified Path"),
            None,
        ));
    }

    let path = req.path.unwrap();
    let mut path_iter = path.split('/');
    path_iter.next();
    match (path_iter.next(), path_iter.next(), path_iter.next()) {
        (Some("clientes"), Some(id), Some(resource)) => {
            let id = id.parse::<u8>();
            if id.is_err() {
                return Err(Response::new(404, "Not Found", Some("Invalid ID"), None));
            }
            let id = id.unwrap();
            if id >= 6 {
                return Err(Response::new(
                    404,
                    "Not Found",
                    Some("ID greater than 6"),
                    None,
                ));
            }
            let method = match req.method {
                Some("GET") => Method::Get,
                Some("POST") => Method::Post,
                _ => {
                    return Err(Response::new(
                        405,
                        "Not Found",
                        Some("Method Not Allowed"),
                        None,
                    ))
                }
            };
            match resource {
                "transacoes" => {
                    if method != Method::Post {
                        return Err(Response::new(405, "Method Not Allowed", None, None));
                    }

                    let buffer = std::str::from_utf8(buffer[nbytes..].into());
                    if let Err(e) = buffer {
                        eprintln!("Backend: could not parse: {e}");
                        return Err(Response::new(
                            422,
                            "Unprocessable Content ",
                            Some("Invalid UTF8 char"),
                            None,
                        ));
                    }
                    let buffer = buffer.unwrap();

                    //  BEGIN OF SERDE WORKAROUND
                    //      Serde does not allow buffers with trailling chars.
                    let mut sbuffer = String::new();
                    for ch in buffer.chars() {
                        if ch == '\0' {
                            break;
                        }
                        sbuffer.push(ch);
                    }
                    sbuffer.shrink_to_fit();
                    //  END OF SERDE WORKAROUND

                    let transacao_body = serde_json::from_str(&sbuffer);
                    if let Err(e) = transacao_body {
                        eprintln!("Backend: Invalid json: {e}");
                        return Err(Response::new(
                            422,
                            "Unprocessable Content",
                            Some("Invalid json"),
                            None,
                        ));
                    }
                    let transacao_body = transacao_body.unwrap();
                    Ok(Paths::Transacao(id, transacao_body))
                }
                "extrato" => {
                    if method != Method::Get {
                        return Err(Response::new(405, "Method Not Allowed", None, None));
                    }
                    Ok(Paths::Extrato(id))
                }
                _ => Err(Response::new(
                    404,
                    "Not Found",
                    Some("Invalid resource"),
                    None,
                )),
            }
        }
        _ => Err(Response::new(404, "Not Found", Some("Invalid path"), None)),
    }
}

fn process_transacao(transacao: TrasacaoBackend, id: u8) -> Response {
    if transacao.descricao.chars().count() > NCHAR_DESCRIPTION {
        return Response::new(
            422,
            "Unprocessable Content",
            Some("Invalid descricao (> 10). Is there graphemes?"),
            None,
        );
    }

    if transacao.tipo.chars().count() > 1 {
        return Response::new(
            422,
            "Unprocessable Content",
            Some("Invalid tipo (> 1)"),
            None,
        );
    }

    let value: i64 = match transacao.tipo.chars().next() {
        Some('d') => -(transacao.valor as i64),
        Some('c') => transacao.valor as i64,
        _ => return Response::new(422, "Unprocessable Content", Some("Invalid tipo"), None),
    };

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH);
    if let Err(e) = timestamp {
        eprintln!("Backend: Timestamp error: {}", e);
        return Response::new(500, "Internal Error", Some("Timestamp error"), None);
    }
    let timestamp = timestamp.unwrap();
    let mut op = Operation {
        kind: OperationKind::Transacao,
        id,
        transacao: Transacao {
            value,
            transacao_description: ['\0'; NCHAR_DESCRIPTION],
            timestap: timestamp.as_secs(),
        },
    };
    for (indx, char) in transacao.descricao.chars().enumerate() {
        op.transacao.transacao_description[indx] = char;
    }

    let op_ptr = unsafe { &*(&op as *const _ as *const BufferOperation) };
    let mut db = DBconn::new();
    for _ in 0..3 {
        if let Ok(_) = db {
            break;
        }
        db = DBconn::new();
    }
    if db.is_err() {
        return Response::new(500, "Internal Error", Some("Cannot connect to DB"), None);
    }
    let db = db.unwrap();
    let dbconn = db.dbconn();
    match dbconn.send(op_ptr) {
        Ok(nbytes) => {
            if nbytes != SIZE_OPERATION {
                eprintln!("Backend: DB send error: Send {nbytes} instead of {SIZE_OPERATION}");
                return Response::new(
                    500,
                    "Internal Error",
                    Some("Backend send wrong bytes"),
                    None,
                );
            }
        }
        Err(e) => {
            eprintln!("Backend: DB send error: {}", e);
            return Response::new(500, "Internal Error", Some("Backend send"), None);
        }
    };
    let mut buff = [0; SIZE_TRANSACAO_RETURN];
    let res = dbconn.recv(&mut buff);
    match res {
        Err(e) => {
            eprintln!("Backend: recv error: {}", e);
            Response::new(500, "Internal Error", Some("Backend recv"), None)
        }
        Ok(nbytes) => {
            if nbytes == 1 {
                return Response::new(
                    422,
                    "Unprocessable Content",
                    Some("Backend DB inform"),
                    None,
                );
            }
            if nbytes != SIZE_TRANSACAO_RETURN {
                return Response::new(
                    500,
                    "Internal Error",
                    Some("Backend DB returns invalid"),
                    None,
                );
            }

            let tret = unsafe { *(&buff as *const _ as *const TransacaoReturn) };
            let sret = serde_json::to_string(&tret);
            if let Err(e) = sret {
                eprintln!("Backend: could not convert db response: {e}");
                return Response::new(
                    500,
                    "Internal Error",
                    Some("Backend convert DB returns error"),
                    None,
                );
            }
            let mut sret = sret.unwrap();
            sret.push_str("\r\n");
            sret.shrink_to_fit();

            Response::new(200, "OK", None, Some(sret.as_bytes().into()))
        }
    }
}

fn process_extrato(id: u8) -> Response {
    let op = Operation {
        kind: OperationKind::Extrato,
        id,
        transacao: unsafe { std::mem::zeroed() },
    };
    let op_ptr = unsafe { &*(&op as *const _ as *const BufferOperation) };
    let mut db = DBconn::new();
    for _ in 0..3 {
        if let Ok(_) = db {
            break;
        }
        db = DBconn::new();
    }
    if db.is_err() {
        return Response::new(500, "Internal Error", Some("Cannot connect to DB"), None);
    }
    let db = db.unwrap();
    let dbconn = db.dbconn();
    match dbconn.send(op_ptr) {
        Ok(nbytes) => {
            if nbytes != SIZE_OPERATION {
                eprintln!("Backend: DB send error: Send {nbytes} instead of {SIZE_OPERATION}");
                return Response::new(
                    500,
                    "Internal Error",
                    Some("Backend send wrong bytes"),
                    None,
                );
            }
        }
        Err(e) => {
            eprintln!("Backend: DB send error: {}", e);
            return Response::new(500, "Internal Error", Some("Backend send"), None);
        }
    };
    let mut buff = [0; SIZE_EXTRATO];
    match dbconn.recv(&mut buff) {
        Err(e) => {
            eprintln!("Backend: recv error: {}", e);
            Response::new(500, "Internal Error", Some("Backend recv"), None)
        }
        Ok(nbytes) => {
            if nbytes == 1 {
                return Response::new(404, "Not Found", None, None);
            }
            if nbytes != SIZE_EXTRATO {
                return Response::new(
                    500,
                    "Internal Error",
                    Some("Backend DB returns invalid"),
                    None,
                );
            }

            let tret = unsafe { *(&buff as *const _ as *const Extrato) };
            let extrato = ExtratoBackend::from(tret);
            let sret = serde_json::to_string(&extrato);
            if let Err(e) = sret {
                eprintln!("Backend: could not convert db response: {e}");
                return Response::new(
                    500,
                    "Internal Error",
                    Some("Backend convert DB returns error"),
                    None,
                );
            }
            let mut sret = sret.unwrap();
            sret.push_str("\r\n");
            sret.shrink_to_fit();
            Response::new(200, "OK", None, Some(sret.as_bytes().into()))
        }
    }
}

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], PORT)))?;
    loop {
        let mut buf = [0; DATA_LIMIT];
        let result = socket.recv_from(&mut buf);
        if let Err(e) = result {
            eprintln!("Backend: couldn't recieve a data: {}", e);
            continue;
        }
        let (nbytes, addr) = result.unwrap();
        let buf = &(buf[..nbytes]);

        println!("Backend: I recived a connection!");

        let path = req_parser(buf);
        if let Err(err_response) = path {
            send_buffer(&socket, &err_response.into_vec(), &addr);
            continue;
        }
        let path = path.unwrap();
        let response: Response = match path {
            Paths::Transacao(id, transacao) => process_transacao(transacao, id),
            Paths::Extrato(id) => process_extrato(id),
        };
        send_buffer(&socket, &response.into_vec(), &addr);
    }
}
