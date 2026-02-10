# 📡 SIP Core - Protokol Matbaası

**Rol:** SIP ve SDP mesajlarını oluşturan ve ayrıştıran (Parse) dilsiz işçi.

## 1. Temel Sorumluluklar

1.  **SDP Oluşturma (`SdpBuilder`):**
    *   Dışarıdan verilen kodek listesini, IP ve Port bilgilerini standart RFC formatında metne döker.
    *   `ptime` ve `rtcp` gibi özellikleri parametre olarak alır.
    *   **Kendi fikri yoktur.** "G.729 ekleyeyim mi?" diye sormaz, ekle denirse ekler.

2.  **SIP Manipülasyonu:**
    *   `Via`, `Contact`, `Record-Route` başlıklarını okur ve yazar.
    *   NAT arkasındaki IP'leri düzeltir (Fix NAT).

## 2. Yasaklar (Anti-Patterns)

*   ❌ **Konfigürasyon Tutmaz:** Hangi kodeğin öncelikli olduğunu bilmez. (Bunu `rtp-core` bilir).
*   ❌ **Durum Tutmaz (Stateless):** Bir çağrının geçmişini bilmez, sadece o anki paketi işler.