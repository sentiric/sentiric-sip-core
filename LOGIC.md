# 🧬 SIP Core Diagnostic & Testing Logic

Bu belge, Sentiric SIP çekirdeğinin (Core) doğruluğunu, yönlendirme mantığını ve ağ seviyesindeki durumunu test etme prensiplerini içerir.

## 1. Canlı Ağ Analizi (Packet Sniffing)
Çekirdek kütüphanenin `Via`, `Record-Route` ve `Contact` başlıklarını doğru manipüle edip etmediğini denetlemek için Kernel seviyesinde ağ analizi kullanılır.

**Sinyalleşme Analizi:**
```bash
sudo tcpdump -i any -n -s 0 -A udp port 5060 | grep -E "SIP/2.0|CSeq:|Call-ID|Contact" --line-buffered
```

**RTP (Medya) Trafiği Doğrulaması:**
```bash
sudo tcpdump -i any -n udp portrange 10000-20000
```

## 2. Sorun Giderme Mantığı (Troubleshooting)
SIP Core, hataları spesifik state machine durumlarına göre loglar.

| Belirti | Kök Neden (Root Cause) | Çözüm (Logic Fix) |
| :--- | :--- | :--- |
| **Sürekli INVITE (Retransmission)** | `200 OK` istemciye ulaşmıyor. | `Contact` başlığındaki IP'nin NAT arkasında ezilip ezilmediğini (Topology Hiding) kontrol et. |
| **ACK Gelmiyor** | İstemci `Record-Route` zincirini takip edemiyor. | `SipRouter::add_record_route` mantığını denetle. |
| **Ses Yok (One-way Audio)** | SDP içindeki `c=IN IP4` adresi yanlış. | `SdpManipulator::rewrite_connection_info` işlevinin çalıştığından emin ol. |