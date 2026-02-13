# 📡 Sentiric SIP Core (v1.5.1)

[![Status](https://img.shields.io/badge/status-production-green.svg)]()
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)]()

**Sentiric SIP Core**, Sentiric telekomünikasyon altyapısının temel taşıdır. RFC 3261 uyumlu SIP paketlerinin oluşturulması, ayrıştırılması (parsing) ve yönlendirme mantığı (routing logic) için gereken tüm araçları sağlar.

**Felsefe:** "Stateless, Safe, Zero-Allocation where possible."

## 🚀 Özellikler

1.  **Strict Parsing:** Hatalı SIP paketlerini ağ geçidinde (Edge) tespit eder ve reddeder.
2.  **Routing Helpers:** `detect_loop`, `decrement_max_forwards` gibi güvenlik kontrollerini standartlaştırır.
3.  **Topology Hiding:** SBC servisleri için `apply_topology_hiding` ile güvenli Contact header manipülasyonu sağlar.
4.  **Transaction Safety:** `SipTransaction` yapısı ile paketlerin durumunu (State) takip etmeyi kolaylaştırır.
5.  **Builder Pattern:** `SipResponseFactory` ve `SipRouter` ile güvenli paket oluşturma.

## 📦 Kurulum

Bu kütüphane, Sentiric ekosistemindeki diğer servisler (`proxy`, `sbc`, `b2bua`, `media`) tarafından temel bağımlılık olarak kullanılır.

```toml
[dependencies]
sentiric-sip-core = { git = "https://github.com/sentiric/sentiric-sip-core.git", tag = "v1.5.1" }
```s