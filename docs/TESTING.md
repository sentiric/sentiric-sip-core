# Sentiric SIP/VoIP Test Rehberi

Bu belge, Sentiric SIP sunucusunun (UAS) doğruluğunu ve performansını test etmek için kullanılan yöntemleri tanımlar.

## 1. Otomatik Testler (UAC - User Agent Client)
Geliştirme sürecinde en hızlı ve güvenilir yöntemdir. Operatöre ihtiyaç duymadan, kendi yazdığımız `sentiric-sip-uac` aracı ile sistemi test ederiz.

### Kullanım
1.  **Sunucuyu Başlat (Terminal 1):**
    ```bash
    cd sentiric-sip-uas
    export PUBLIC_IP="127.0.0.1"
    cargo run --release
    ```

2.  **İstemciyi Başlat (Terminal 2):**
    ```bash
    cd sentiric-sip-uac
    # Sunucu IP'sini parametre olarak ver
    cargo run --release -- 127.0.0.1
    ```

### Beklenen Çıktı
*   [UAC] INVITE gönderildi.
*   [UAS] INVITE alındı, 100 Trying gönderildi.
*   [UAS] 200 OK gönderildi.
*   [UAC] 200 OK alındı, **ACK gönderildi**.
*   [UAS] **ACK alındı**. (En kritik adım)
*   [UAC] RTP (Ses) paketleri almaya başladı.

---

## 2. Canlı Ağ Analizi (tcpdump)
Operatör (Sippy) entegrasyonu sırasında veya şüpheli durumlarda, ağ trafiğini "dinlemek" için kullanılır.

### Komutlar
Sadece SIP sinyalleşmesini (metin tabanlı) görmek için:
```bash
# -n: IP çözümleme yapma (hızlandırır)
# -s 0: Paketin tamamını yakala
# -A: ASCII (metin) olarak göster
# --line-buffered: Anlık ekrana bas
sudo tcpdump -i any -n -s 0 -A udp port 5060 | grep -E "SIP/2.0|CSeq:|Call-ID|Contact" --line-buffered
```

RTP (Ses) trafiğinin akıp akmadığını görmek için (Sadece paket sayısını izler):
```bash
sudo tcpdump -i any -n udp portrange 10000-20000
```

---

## 3. Uygulama İçi Loglama
Kodumuz (`sip-core` ve `sip-uas`), kritik olayları standart çıktıda (stdout) loglar.

*   `[INFO]`: Normal akış (Çağrı geldi, cevaplandı, kapandı).
*   `[WARN]`: Potansiyel sorunlar (Retransmission algılandı).
*   `[ERROR]`: Kritik hatalar (Soket hatası, Parse hatası).

**Örnek Başarılı Akış Logu:**
```text
[INFO ] [CALL] INVITE Arayan: <sip:555@...>, Hedef: ...
[INFO ] [SIP ] 200 OK Yanıt gönderildi
[INFO ] [SIP ] ACK ALINDI! Bağlantı başarılı.
[INFO ] [RTP ] Ses akışı başlıyor...
```

---

## 4. Sorun Giderme (Troubleshooting)

| Belirti | Olası Neden | Çözüm |
| :--- | :--- | :--- |
| **Sürekli INVITE geliyor (Retransmission)** | Sunucu yanıtı (`200 OK`) karşı tarafa ulaşmıyor veya hatalı. | Firewall'u kontrol et. `Contact` başlığında Public IP olduğundan emin ol. |
| **ACK gelmiyor** | `Contact` başlığı formatı Sippy'nin beklediği gibi değil. | `Record-Route` başlıklarını kaldır. `Contact` içine kullanıcı adını ekle. |
| **Ses yok (RTP gitmiyor)** | RTP portları (10000-20000) kapalı veya NAT sorunu. | `gateway-setup.sh` ile port yönlendirmeyi kontrol et. |

