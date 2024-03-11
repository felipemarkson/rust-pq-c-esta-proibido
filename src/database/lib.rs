use std::marker::PhantomData;
use std::mem::{self, size_of};
use std::net::{SocketAddr, UdpSocket};
use std::ptr::{read, write};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

pub const SIZE_OPERATION: usize = size_of::<Operation>();
pub const SIZE_EXTRATO: usize = size_of::<Extrato>();
pub const SIZE_TRANSACAO_RETURN: usize = size_of::<TransacaoReturn>();
pub const NCHAR_DESCRIPTION: usize = 10; // 10 chars + \0
pub const NTRANSACOES: usize = 10; // 10 chars + \0

pub type BufferOperation = [u8; SIZE_OPERATION];
pub type BufferDescription = [char; NCHAR_DESCRIPTION];
pub type BufferExtrato = [u8; SIZE_EXTRATO];
pub type BufferTranscaoReturn = [u8; SIZE_TRANSACAO_RETURN];
pub const PORT_DB: u16 = 7000;

pub const RES_ERROR: [u8; 1] = [1];

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Transacao {
    pub value: i64,
    pub transacao_description: BufferDescription,
    pub timestap: SystemTime,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TransacaoExtrato {
    pub isvalid: bool,
    pub value: i64,
    pub transacao_description: BufferDescription,
    pub timestap: SystemTime,
}
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Extrato {
    pub total: i64,
    pub limite: i64,
    pub transacoes: [TransacaoExtrato; NTRANSACOES],
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(C, packed)]
pub struct TransacaoReturn {
    pub limite: i64,
    pub saldo: i64,
}

#[repr(C)]
pub enum OperationKind {
    Extrato,
    Transacao,
}

#[repr(C, packed)]
pub struct Operation {
    pub kind: OperationKind,
    pub id: u8,
    pub transacao: Transacao,
}

pub struct Converter<T, B: Copy>(PhantomData<T>, PhantomData<B>);
impl<T, B: Copy> Converter<T, B> {
    const _SIZE_OK_: () = assert!(size_of::<T>() == size_of::<B>());
    pub fn from_buffer(buffer: &B) -> T {
        unsafe { read(buffer as *const _ as *const T) }
    }
    pub fn to_buffer(op: &T) -> B {
        unsafe {
            let mut buffer: B = mem::zeroed();
            write(&mut buffer, *(op as *const _ as *const B));
            buffer
        }
    }
}

pub struct DBconn {
    dbconn: UdpSocket,
}

impl DBconn {
    pub fn new() -> Result<DBconn, ()> {
        for port in (PORT_DB + 1)..(PORT_DB + 1000) {
            let dbconn = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], port)));
            if let Ok(dbconn) = dbconn {
                if let Err(_) = dbconn.connect(SocketAddr::from(([127, 0, 0, 1], PORT_DB))) {
                    continue;
                }
                return Ok(DBconn { dbconn });
            }
        }
        Err(())
    }
    pub fn dbconn(&self) -> &UdpSocket {
        &self.dbconn
    }
}
