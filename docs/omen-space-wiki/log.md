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

---

## [2026-09-29] Feature | Victus Max Proaktif Better Auto ve Bağımsız Depo Geçişi

- **İşlem Türü:** Özellik Ekleme, Algoritma Portu ve Mimari Yeniden Yapılandırma
- **Kapsam:** `victus-max-daemon`, `victus-max-cli`, `victus-max-gui`, `victus-max-types`, `victus-max-tray`, `victus-max-overlay` ve sistem birimleri.
- **Detaylar:**
  - `victus-control` projesinin proaktif iş yükü algısı (`/proc/stat` delta CPU yükü) `BetterAutoEngine` ile Rust çekirdeğine port edildi.
  - Balanced modu için hem CPU hem GPU fanında varsayılan asgari devir **2600 RPM** olarak belirlendi; kullanıcıya 2000–3500 RPM ayar aralığı sağlandı.
  - Balanced modunda fan sesini dizginlemek için ayarlanabilir Akustik Tavan (Acoustic Ceiling, Seviye 3–8, varsayılan Seviye 5 / ~4100 RPM) mekanizması entegre edildi.
  - Donanım güvenliği: HP Victus EC veri yolu kilitlenmesini engellemek için Fan 1 ve Fan 2 yazımları arasına 10 saniyelik asenkron bekleme (stagger gap) ve 90 saniyelik BIOS watchdog tazelemesi yerleştirildi.
  - 88°C acil durum termal baypası ile sıcaklık fırladığında tavan devreden çıkarılarak tam soğutma sağlandı.
  - D-Bus `org.hp.omen.Fan` sözleşmesi yeni yöntemlerle genişletildi; GTK4 GUI ve CLI komutları eklendi.
  - Proje tekil bağımsız kök depoya (`/home/umutaktepe/victus-max`) taşındı, ikili dosyalar `victus-max*` olarak adlandırıldı.
- **Mimari Karar:** [[adr-005-better-auto-proactive-fan-and-victus-max]]
- **Test ve Doğrulama:** 46 birim ve entegrasyon testi eksiksiz geçti (`cargo test --workspace`).

