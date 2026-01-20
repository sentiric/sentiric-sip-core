// sentiric-sip-core/src/transport.rs

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use crate::error::SipError;

pub struct SipTransport {
    socket: Arc<UdpSocket>,
    buf_size: usize,
}

impl SipTransport {
    pub async fn new(bind_addr: &str) -> Result<Self, SipError> {
        let socket = UdpSocket::bind(bind_addr).await
            .map_err(|e| SipError::NetworkError(format!("Bind hatası: {}", e)))?;
        
        Ok(Self {
            socket: Arc::new(socket),
            buf_size: 65535, // Standart UDP MTU üstü güvenli alan
        })
    }
    
    /// Mevcut bir soketi kullanarak transport oluşturur (Paylaşılan soketler için)
    pub fn from_socket(socket: Arc<UdpSocket>) -> Self {
        Self {
            socket,
            buf_size: 65535,
        }
    }

    /// Bir sonraki SIP paketini bekler.
    /// Keep-alive (CRLF) paketlerini otomatik olarak filtreler.
    pub async fn recv(&self) -> Result<(Vec<u8>, SocketAddr), SipError> {
        let mut buf = vec![0u8; self.buf_size];
        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    // Temel Validasyon: Çok kısa veya sadece ping (CRLF) paketlerini atla
                    if len < 4 || buf[..len].iter().all(|&b| b == b'\r' || b == b'\n' || b == 0) {
                        continue;
                    }
                    return Ok((buf[..len].to_vec(), addr));
                }
                Err(e) => return Err(SipError::NetworkError(e.to_string())),
            }
        }
    }

    pub async fn send(&self, data: &[u8], target: SocketAddr) -> Result<(), SipError> {
        self.socket.send_to(data, target).await
            .map_err(|e| SipError::NetworkError(e.to_string()))?;
        Ok(())
    }
    
    pub fn get_socket(&self) -> Arc<UdpSocket> {
        self.socket.clone()
    }
}