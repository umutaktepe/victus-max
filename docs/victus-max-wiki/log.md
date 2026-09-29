# Victus Max Wiki Kronolojik İşlem Günlüğü (Log)

Bu dosya, Victus Max LLM Wiki bilgi tabanında gerçekleştirilen tüm modelleme, kaynak işleme (ingest), sorgulama (query) ve doğrulama (lint) adımlarının zamana göre sıralı (append-only) kayıt defteridir.

Kayıtlar standart Unix komutlarıyla filtrelenebilir formatta tutulur:
```bash
grep "^## \[" docs/victus-max-wiki/log.md | tail -5
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

---

## [2026-09-29] Refactor | Wiki Dizin Yeniden Adlandırma (victus-max-wiki)

- **İşlem Türü:** Dokümantasyon ve Wiki Dizin Yeniden Yapılandırması
- **Kapsam:** `docs/victus-max-wiki/`, `AGENTS.md`.
- **Detaylar:**
  - `docs/omen-space-wiki/` dizini `docs/victus-max-wiki/` olarak taşındı.
  - `AGENTS.md` (LLM Wiki Sözleşmesi) kuralları, başlığı ve yol referansları güncellendi.
  - Wiki MOC (`index.md`) ve işlem günlüğü (`log.md`) başlık ve yol komutları Victus Max kimliğiyle uyumlu hale getirildi.
- **Sistem Durumu:** Kararlı, graf bağlantıları ve Obsidian wikilink'leri eksiksiz korunuyor.

---

## [2026-09-29] Feature | Victus Max Overlay (HUD) Modernizasyonu ve Better Auto Entegrasyonu

- **İşlem Türü:** Arayüz İyileştirme, Vektör Varlık Çözümleme ve HUD Genişletmesi
- **Kapsam:** `src/victus-max-overlay/`, `src/victus-max-gui/src/asset_resolver.rs`, `docs/victus-max-wiki/user-experience/quick-hud-overlay.md`.
- **Detaylar:**
  - Shift+F2 HUD başlığı `"VICTUS MAX HIZLI KONTROL"` olarak güncellendi ve yeni Victus Max hibrit logosu başlığa entegre edildi.
  - Masaüstü ortamı / simge teması bağımsızlığı: Kırık kırmızı kutulara yol açan GNOME bağımlı sembolik simgeler yerine yerel SVG vektörleri (`eco.svg`, `balanced.svg`, `performance.svg`, `better_auto.svg`, `custom.svg`) doğrudan dosya yoluyla yüklendi.
  - Fan modları 4 karta genişletildi (pencere genişliği 720px'e çıkarıldı):
    - `[Q] Better Auto` (Turuncu/kırmızı neon vurgu)
    - `[W] Otomatik`
    - `[E] Maksimum`
    - `[R] Özel`
  - Klavye kısayolları ve alt bilgi etiketleri Q/W/E/R ile uyumlu hale getirildi.
- **Test ve Doğrulama:** `cargo check --workspace` ve `cargo test --workspace` (46 test) hatasız tamamlandı.

---

## [2026-09-29] Branding & Polish | Kapsamlı Victus Max İsim Arındırması ve Tray Entegrasyonu

- **İşlem Türü:** Marka Arındırma, Sistem Tepsisi Genişletmesi ve Yapılandırma Yolu Göçü
- **Kapsam:** `src/victus-max-tray/`, `src/victus-max-gui/`, `src/victus-max-daemon/`, `src/victus-max-cli/`, `setup.sh`, `install.sh`, `PKGBUILD`.
- **Detaylar:**
  - Sistem tepsisi (`victus-max-tray`) sağ tık menüsündeki `"OMENSpace'i Aç"` ibaresi `"Victus Max'ı Aç"` (ve İngilizce `"Open Victus Max"`) olarak değiştirildi.
  - Tepsi fan modları menüsüne `Better Auto` seçeneği eklendi; tooltip metnine `Better Auto` entegre edildi.
  - Daemon masaüstü bildirimleri (`notifier.rs`, `fan_cleaning.rs`, `game_automation.rs`, `hid_wizard.rs`, `platform.rs`, `auto_updater.rs`, `bios_checker.rs`) ve donanım raporları Victus Max kimliğine kavuşturuldu; GitHub issue bağlantıları `umutaktepe/victus-max` reposuna yönlendirildi.
  - Yapılandırma yolları öncelikli olarak `~/.config/victus-max/` ve `/etc/victus-max/` adreslerini kullanacak şekilde güncellendi; mevcut kullanıcılar için `~/.config/omenspace/` ve `/etc/omenspace/` yollarına tam geriye dönük uyumluluk (fallback) korundu.
  - GUI uygulama kimliği (`APP_ID`) `org.hp.VictusMax` olarak ayarlandı, `org.hp.VictusMax.service` D-Bus aktivasyon birimi oluşturuldu ve `setup.sh` ile kurulum adımlarına eklendi.
  - Kurulum (`setup.sh`, `install.sh`) ve paketleme (`PKGBUILD`) betikleri Victus Max ikili dosyalarını önceliklendirecek ve eski `omen-*` isimlerine sembolik bağlar sunacak şekilde güncellendi.
- **Test ve Doğrulama:** `cargo check --workspace` ve `cargo test --workspace` (46 test) hatasız tamamlandı.
