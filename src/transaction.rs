// sentiric-sip-core/src/transaction.rs

use crate::packet::{SipPacket, Method};
use crate::header::HeaderName;

/// RFC 3261 Transaction States
/// (Basitleştirilmiş UAS INVITE/NON-INVITE State Machine)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// Başlangıç durumu
    Null,
    /// İstek alındı, işleniyor (100 Trying gönderildi)
    Proceeding,
    /// Nihai yanıt (2xx, 3xx-6xx) üretildi
    Completed,
    /// ACK alındı (Sadece Invite işlemleri için)
    Confirmed,
    /// İşlem sonlandı
    Terminated,
}

/// Transaction Identifier (Branch ID + Method + CSeq)
/// Bir işlemi benzersiz kılan anahtar.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionId {
    pub branch: String,
    pub method: Method,
    pub cseq: u32,
}

impl TransactionId {
    pub fn new(branch: &str, method: Method, cseq: u32) -> Self {
        Self {
            branch: branch.to_string(),
            method,
            cseq,
        }
    }
}

/// Transaction Context
/// Bir SIP işleminin anlık fotoğrafı.
#[derive(Debug, Clone)]
pub struct SipTransaction {
    pub id: TransactionId,
    pub state: TransactionState,
    pub original_request: SipPacket,
    pub last_response: Option<SipPacket>,
}

impl SipTransaction {
    /// Gelen bir istekten yeni bir Transaction başlatır.
    pub fn new(req: &SipPacket) -> Option<Self> {
        // 1. Branch ID'yi Via başlığından çek
        let branch = req.headers.iter()
            .find(|h| h.name == HeaderName::Via)
            .and_then(|h| {
                h.value.split("branch=")
                    .nth(1)
                    .map(|s| s.split_whitespace().next().unwrap_or(""))
                    .map(|s| s.split(';').next().unwrap_or(""))
                    .map(|s| s.trim_matches(',')) // Bazı clientlar virgül ekleyebilir
            })?;

        // 2. CSeq numarasını çek
        let cseq_val = req.get_header_value(HeaderName::CSeq)?;
        let cseq_num = cseq_val.split_whitespace().next()?.parse::<u32>().ok()?;

        let id = TransactionId::new(branch, req.method.clone(), cseq_num);

        Some(Self {
            id,
            state: TransactionState::Proceeding, // Varsayılan olarak işlemeye başlıyoruz
            original_request: req.clone(),
            last_response: None,
        })
    }

    /// Dışarıya gönderilen bir yanıta göre durumu günceller.
    pub fn update_on_response(&mut self, resp: &SipPacket) {
        if resp.status_code >= 100 && resp.status_code < 200 {
            self.state = TransactionState::Proceeding;
        } else if resp.status_code >= 200 && resp.status_code < 300 {
            // 2xx yanıtlarında transaction hemen sonlanır (ACK, yeni bir transaction başlatmaz)
            // Ancak retransmission için completed gibi davranıp yanıtı saklamalıyız.
            self.state = TransactionState::Terminated; 
        } else if resp.status_code >= 300 {
            // Hata yanıtları için ACK beklenir
            self.state = TransactionState::Completed;
        }
        
        self.last_response = Some(resp.clone());
    }
}

/// Uygulama katmanının ne yapması gerektiğini söyleyen sonuç enum'ı.
pub enum TransactionAction {
    /// Bu yeni bir istektir, uygulama (Business Logic) işlemelidir.
    ForwardToApp,
    /// Bu bir tekrardır (Retransmission). Uygulama işlememeli,
    /// Core katmanı verilen paketi (cached_response) ağa geri basmalıdır.
    RetransmitResponse(SipPacket),
    /// Bu bir tekrar ama henüz yanıt üretilmedi. Yoksay.
    Ignore,
}

pub struct TransactionEngine;

impl TransactionEngine {
    /// Gelen paketi mevcut transaction durumuyla karşılaştırır ve aksiyon önerir.
    pub fn check(
        transaction: &Option<SipTransaction>,
        packet: &SipPacket
    ) -> TransactionAction {
        // Durum yoksa (İlk paket), uygulama işlemelidir.
        if transaction.is_none() {
            return TransactionAction::ForwardToApp;
        }

        let tx = transaction.as_ref().unwrap();

        // 1. Retransmission Kontrolü (INVITE tekrarı mı?)
        // Method ve Branch ID kontrolü (Basitlik için burada Method kontrolü yeterli varsayıyoruz, 
        // çünkü Session içinde zaten Call-ID ile eşleştik).
        if packet.method == tx.id.method {
            // Eğer elimizde hazır bir yanıt varsa, onu tekrar gönder (RETRANSMISSION)
            if let Some(cached_resp) = &tx.last_response {
                return TransactionAction::RetransmitResponse(cached_resp.clone());
            }
            // Yanıt henüz üretilmediyse (Hala Processing), isteği yoksay (Merge requests)
            return TransactionAction::Ignore;
        }

        // 2. ACK Kontrolü
        // ACK, INVITE transaction'ının bir parçası değildir ama onun sonucudur.
        // Uygulama katmanı ACK'i alıp dialog durumunu 'Established' yapmalıdır.
        if packet.method == Method::Ack {
            return TransactionAction::ForwardToApp;
        }

        // Diğer durumlar (Örn: CANCEL)
        TransactionAction::ForwardToApp
    }
}