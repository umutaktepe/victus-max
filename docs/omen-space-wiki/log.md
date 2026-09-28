# OMEN Space Wiki Kronolojik İşlem Günlüğü (Log)

Bu dosya, OMEN Space LLM Wiki bilgi tabanında gerçekleştirilen tüm modelleme, kaynak işleme (ingest), sorgulama (query) ve doğrulama (lint) adımlarının zamana göre sıralı (append-only) kayıt defteridir.

Kayıtlar standart Unix komutlarıyla filtrelenebilir formatta tutulur:
```bash
grep "^## \[" docs/omen-space-wiki/log.md | tail -5
```

---

## [2026-09-29] Init | OMEN Space Karpathy LLM Wiki Ontolojisi ve Yaşayan Mimari Kurulumu

- **İşlem Türü:** İlk Sistem Kurulumu ve Dinamik Ontolojik Modelleme
- **Kapsam:** Kod tabanının tamamı (6 Rust workspace sandığı: `omen-space-daemon`, `omen-gui`, `omen-cli`, `omen-overlay`, `omen-tray`, `omen-types`; 2 Linux çekirdek sürücüsü: `hp-omen-extra.c`, `hp-wmi.c`; sistem verileri: D-Bus, Udev, Systemd, Flakes, PKGBUILD).
- **Domain Yapısı:** 6 fonksiyonel mimari rol belirlendi ve klasörlendi:
  1. `decisions/`: Mimari Karar Kayıtları (ADR).
  2. `hardware-drivers/`: Çekirdek sürücüleri, ACPI, EC, MSR ve SMU.
  3. `system-daemon/`: Daemon yaşam döngüsü ve 10+ arka plan mikroservisi.
  4. `interfaces/`: D-Bus IPC sözleşmeleri, telemetri şemaları ve donanım matrisi (`boards.json`).
  5. `user-experience/`: GTK4/Libadwaita arayüzü, Wayland HUD overlay, CLI ve tepsi uygulaması.
  6. `platform-integration/`: Systemd, Polkit, Udev izinleri ve DKMS dağıtımı.
- **Mimari Kararlar (ADR):**
  - [[adr-001-rust-daemon-client-split]]: Ayrık yetkilendirme modeli (root daemon vs unprivileged GTK4 client).
  - [[adr-002-wmi-vs-direct-ec-arbitration]]: Güvensiz anakartlarda Caps Lock panik engelleme ve hibrit EC/WMI tahkimi.
  - [[adr-003-companion-dkms-driver-non-clashing]]: WMI GUID'i sahiplenmeden stock `hp-wmi` ile birlikte yaşayan yoldaş DKMS sürücüsü.
  - [[adr-004-native-msr-and-smu-mailbox-tuning]]: Dış CLI araçlarına ihtiyaç duymayan yerel MSR/SMU bit paketleme mimarisi.
- **Graf Hijyeni ve Ağ Topolojisi:**
  - Sayısal sıralama ("01-", "02-") tamamen elendi.
  - Tüm bağlantılar Obsidian uyumlu `[[dosya-adi]]` wikilink sözdizimiyle doğrudan bağımlı modüllere kuruldu.
  - Toplam 42 atomik dokümantasyon sayfası ve ana fihrist ([[index]]) inşa edildi.
- **Sistem Durumu:** Kararlı (v2.1.2), tüm mikroservisler ve veri modelleri eksiksiz belgelendi.
