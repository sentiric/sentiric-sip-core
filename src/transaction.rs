// sentiric-sip-core/src/transaction.rs

use crate::packet::{SipPacket, Method};
use crate::header::HeaderName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Proceeding,
    Completed,
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionId {
    pub branch: String,
    pub method: Method,
}

#[derive(Debug, Clone)]
pub struct SipTransaction {
    pub id: TransactionId,
    pub state: TransactionState,
    pub last_response: Option<SipPacket>,
}

impl SipTransaction {
    pub fn new(req: &SipPacket) -> Option<Self> {
        let branch = req.get_header_value(HeaderName::Via)?
            .split("branch=")
            .nth(1)?
            .split(';')
            .next()?
            .to_string();

        let id = TransactionId {
            branch,
            method: req.method.clone(),
        };

        Some(Self {
            id,
            state: TransactionState::Proceeding,
            last_response: None,
        })
    }

    pub fn update_with_response(&mut self, resp: &SipPacket) {
        if resp.status_code >= 200 {
            self.state = TransactionState::Completed;
        }
        self.last_response = Some(resp.clone());
    }
}

pub enum TransactionAction {
    ForwardToApp,
    Retransmit(SipPacket),
    Ignore,
}

pub struct TransactionEngine;

impl TransactionEngine {
    pub fn check(tx: &Option<SipTransaction>, packet: &SipPacket) -> TransactionAction {
        if let Some(t) = tx {
            if packet.is_request && packet.method == t.id.method {
                if let Some(resp) = &t.last_response {
                    return TransactionAction::Retransmit(resp.clone());
                }
                return TransactionAction::Ignore;
            }
        }
        TransactionAction::ForwardToApp
    }
}