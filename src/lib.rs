// sentiric-sip-core/src/lib.rs

pub mod header;
pub mod packet;
pub mod parser;
pub mod uri;
pub mod utils;
pub mod error;
pub mod profiles;
pub mod transport;
pub mod builder;
pub mod sdp;
pub mod routing; // YENİ EKLENDİ

pub use header::{Header, HeaderName};
pub use packet::{SipPacket, Method, Version};
pub use uri::SipUri;
pub use error::SipError;
pub use profiles::{SipProfile, create_profile};
pub use transport::SipTransport;
// Kolay erişim için routing trait'ini dışarı açıyoruz
pub use routing::SipRouter;