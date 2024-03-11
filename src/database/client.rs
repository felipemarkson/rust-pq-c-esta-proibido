use database::{Extrato, Transacao, TransacaoExtrato, NCHAR_DESCRIPTION, NTRANSACOES};
use std::collections::VecDeque;
use std::mem::size_of;
use std::time::UNIX_EPOCH;

#[derive(Debug)]
pub struct Client {
    pub id: u8,
    pub limite: i64,
    pub saldo: i64,
    pub transacoes: VecDeque<Transacao>,
}
impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl From<&ClientRaw> for Client {
    fn from(client: &ClientRaw) -> Self {
        let mut out = Client {
            id: client.id,
            limite: client.limite,
            saldo: client.saldo,
            transacoes: VecDeque::new(),
        };

        for (indx, transacao) in client.transacoes.iter().enumerate() {
            if indx >= client.ntransacoes {
                break;
            }
            out.transacoes.push_back(*transacao)
        }

        out
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct ClientRaw {
    id: u8,
    limite: i64,
    saldo: i64,
    ntransacoes: usize,
    transacoes: [Transacao; 6],
}

impl From<&Client> for ClientRaw {
    fn from(client: &Client) -> Self {
        let mut out = ClientRaw {
            id: client.id,
            limite: client.limite,
            saldo: client.saldo,
            transacoes: unsafe { std::mem::zeroed() },
            ntransacoes: client.transacoes.len(),
        };
        for (indx, transacao) in client.transacoes.iter().enumerate() {
            out.transacoes[indx] = *transacao;
        }
        out
    }
}

impl Client {
    pub fn new(id: u8, limite: i64, saldo: i64) -> Client {
        let client = Client {
            id,
            limite,
            saldo,
            transacoes: VecDeque::new(),
        };
        println!("DB.Client: Creating new DB for id {id} in /client_{id}.db");
        client.save_client();
        client
    }
    pub fn load_client(id: u8) -> Option<Client> {
        let buff = std::fs::read(format!("client_{}.db", id));
        if let Err(e) = buff {
            eprintln!("DB.Client: Could not load DB from /client_{id}.db: {}", e);
            return None;
        }
        let buff = buff.unwrap();
        if buff.len() != size_of::<ClientRaw>() {
            return None;
        }
        let client: &ClientRaw = unsafe { &*(buff.as_ptr() as *const ClientRaw) };
        println!("DB.Client: Data loaded from /client_{id}.db");
        Some(client.into())
    }
    fn save_client(&self) {
        let client_raw: ClientRaw = self.into();
        let buff: &[u8; size_of::<ClientRaw>()] = unsafe { std::mem::transmute(&client_raw) };
        let _ = std::fs::write(format!("client_{}.db", self.id), buff);
    }
    pub fn push_transacao(&mut self, transacao: Transacao) -> Result<(), ()> {
        if self.saldo + transacao.value < -self.limite {
            return Err(());
        }
        self.saldo += transacao.value;
        if self.transacoes.len() >= 6 {
            self.transacoes.pop_back();
        }
        self.transacoes.push_front(transacao);
        self.save_client();
        Ok(())
    }
    pub fn extrato(&self) -> Extrato {
        let mut transacoes = [TransacaoExtrato {
            isvalid: false,
            value: 0,
            transacao_description: ['\0'; NCHAR_DESCRIPTION],
            timestap: UNIX_EPOCH,
        }; NTRANSACOES];

        for (indx, transacao) in self.transacoes.iter().enumerate() {
            transacoes[indx].isvalid = true;
            transacoes[indx].value = transacao.value;
            transacoes[indx].transacao_description = transacao.transacao_description;
            transacoes[indx].timestap = transacao.timestap;
        }

        Extrato {
            total: self.saldo,
            limite: self.limite,
            transacoes,
        }
    }
}
