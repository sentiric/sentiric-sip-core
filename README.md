# Sentiric SIP Core Library

Bu kütüphane, Sentiric ekosistemi için **RFC 3261** uyumlu, performans odaklı ve bağımlılık içermeyen SIP paket işleyicisidir.

## Özellikler
*   **Zero-Dependency:** `std` dışında hiçbir kütüphane kullanmaz.
*   **Strict Typing:** Header ve Metodlar enum tabanlıdır, hata yapmayı zorlaştırır.
*   **Sippy Compatible:** Sippy Softswitch ve katı firewall kuralları için özel header yönetimi.

## Kullanım
Bu kütüphane tek başına çalışmaz. `sip-uas` veya `sip-uac` projelerine dependency olarak eklenmelidir.