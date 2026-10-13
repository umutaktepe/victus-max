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

---

## [2026-09-29] Docs | Kapsamlı İngilizce Performans ve Soğutma Modları Kılavuzu

- **İşlem Türü:** Kullanıcı ve Geliştirici Dokümantasyonu
- **Kapsam:** `docs/performance-and-cooling-modes.md`.
- **Detaylar:**
  - Tüm performans güç profilleri (`Quiet / Eco`, `Balanced`, `Performance`), ACPI platform_profile, Intel RAPL / AMD RyzenAdj ve NVIDIA Dynamic Boost (PPAB) mimarisi açıklandı.
  - Tüm fan modları (`Better Auto`, `Auto`, `Max Boost`, `Custom`, `Hardware EC`) detaylandırıldı.
  - Better Auto proaktif algoritmasının 8 seviyeli sıcaklık & yük matrisi, yukarı/aşağı histerezis eşikleri, tek adımlı iniş sınırlayıcısı (single-step ramp-down), ayarlanabilir akustik tavan, 88°C acil durum termal baypası, asgari 2600 RPM tabanı ve 10 saniyelik EC veri yolu koruması tablolarla örneklendirildi.
  - 5 gerçek dünya kullanım senaryosu (Ofis/Genel Çalışma, AAA/Espor Oyunculuğu, Sessiz Kütüphane Ortamı, Ağır Derleme & Render, Periyodik Fan Tozu Temizleme) eklendi.
  - CLI, GUI, Shift+F2 HUD ve Sistem Tepsisi hızlı kullanım örnekleri ve özet karşılaştırma matrisi sunuldu.

---

## [2026-09-29] Docs | Kapsamlı Dokümantasyon ve Wiki İsim Arındırması

- **İşlem Türü:** Dokümantasyon Refaktörü ve Marka Standardizasyonu
- **Kapsam:** `docs/` kök rehberleri (`architecture.md`, `backend.md`, `cli.md`, `driver.md`, `gui.md`, `tray.md`, `CODE_REVIEW_GUIDE.md`, `CONTRIBUTING.md`) ve `docs/victus-max-wiki/` bilgi tabanı sayfaları (`adr-001`, `board-capabilities-matrix`, `dbus-ipc-protocol`, `system-telemetry-spec`, `power-dbus-interface`, `ryzen-dbus-interface`, `packaging-and-dkms`, `polkit-dbus-security`, `systemd-services`, `udev-device-rules`, `conflict-detector`, `mux-service`, `power-automation-service`, `power-service`, `rgb-service`, `ryzen-service`, `sysmon-service`, `undervolt-service`, `index.md`).
- **Detaylar:**
  - `docs/` altındaki tüm modül ve mimari kılavuzları Victus Max workspace ve ikili dosya yapısına (`victus-max-*`) uyarlandı.
  - CLI kılavuzundaki eski komutlar `victus-max-cli fan set-mode better-auto`, `set-ceiling`, `set-min-rpm`, `system clean-fans` ve `fetch` komutlarını içerecek şekilde yenilendi.
  - Wiki dokümanlarındaki tüm eski "OMEN Space" ve "omenspace" adlandırmaları "Victus Max" olarak güncellendi; donanım uyumluluğu ve geriye dönük fallback dosya yolları (`/etc/omenspace/` ve `~/.config/omenspace/`) korundu.
  - D-Bus ve mimari belgelerinde `victus-max-types` ve `victus-max-daemon` isimleri tam olarak tutarlı hale getirildi.

---

## [2026-09-29] Fix | Systemd Servis Çakışması ve Kurulum Düzeltmesi

- **İşlem Türü:** Hata Düzeltme & Dağıtım Standardizasyonu
- **Kapsam:** `data/victus-max-daemon.service`, `setup.sh`, `PKGBUILD`.
- **Detaylar:**
  - `data/victus-max-daemon.service` birim dosyasındaki `ExecStart` yolu (`/usr/libexec/victus-max/victus-max-daemon`), `StateDirectory` ve `ReadWritePaths` dizinleri Victus Max'a uyarlandı.
  - `setup.sh` ve `PKGBUILD` içerisinde `omen-space-daemon.service` dosyasının fiziksel bir kopya olarak yüklenmesi durduruldu; `Alias=omen-space-daemon.service` ile systemd sembolik bağına bırakıldı.
  - Kurulum öncesinde var olan eski `omen-space-daemon.service` dosyasının temizlenmesi (`rm -f /etc/systemd/system/omen-space-daemon.service`) sağlanarak `Failed to enable unit: File already exists` hatası giderildi.
  - `setup.sh` sonundaki servis başlatma ve doğrulama testleri `victus-max-daemon` ve `victus-max-cli` olarak güncellendi.

---

## [2026-09-29] Cleanup | Omen-Space Mirasından Tam Arındırma (Decoupling)

- **İşlem Türü:** Kod Tabanı ve Paketleme Temizliği
- **Kapsam:** `data/victus-max-daemon.service`, `data/`, `setup.sh`, `PKGBUILD`, `docs/architecture.md`, `docs/victus-max-wiki/`.
- **Detaylar:**
  - `data/victus-max-daemon.service` dosyasından `Alias=omen-space-daemon.service` kaldırıldı; systemd üzerinde istenmeyen sembolik bağ oluşturulması sonlandırıldı.
  - `data/` altındaki gereksiz `omen-space-daemon.service`, `org.hp.OmenSpace.desktop`, `org.hp.OmenSpace.service` ve `omenspace.png` dosyaları depodan tamamen silindi.
  - `99-omen-space.rules` -> `99-victus-max.rules` ve `sysusers.d/omen-space.conf` -> `sysusers.d/victus-max.conf` olarak yeniden adlandırıldı.
  - `setup.sh` ve `PKGBUILD` içerisinden tüm `omen-cli`, `omen-gui`, `omen-tray`, `omen-overlay`, `omen-space-daemon` sembolik bağları kaldırıldı; kurulum ve kaldırma adımları sistemi eski kırıntılardan tamamen arındıracak şekilde güncellendi.

---

## [2026-09-29] Fix | Fan Presets (Quiet) Custom Moduna Atlama Hatası ve State Senkronizasyonu

- **İşlem Türü:** Hata Düzeltme & UI/Daemon Durum Yönetimi
- **Kapsam:** `src/victus-max-gui/src/performance_control.rs`, `src/victus-max-gui/src/fan_presets.rs`, `src/victus-max-gui/src/daemon_client.rs`, `src/victus-max-daemon/src/config.rs`, `src/victus-max-gui/src/monitoring.rs`, `src/victus-max-daemon/src/main.rs`.
- **Detaylar:**
  - Kullanıcı "Quiet" veya başka bir fan profili seçtiğinde, daemon fan modu `custom` olduğu için 1.5 saniyede bir çalışan arka plan live sync döngüsünün `custom_btn.set_active(true)` çağırarak aktif preset kartını zorla devre dışı bırakması ve curve drawer'ı otomatik açması engellendi.
  - GUI tarafında `preset_buttons` havuzu ve `matches_curve` tolerans karşılaştırıcısı geliştirildi; sistem başlatıldığında veya `custom` moduna geçildiğinde daemon'daki mevcut eğri preset'lerle eşleştirilerek doğru kartın seçili kalması sağlandı.
  - Preset seçildiğinde `cpu_pts` anında güncellenerek eğri çizicinin (`da`) mevcut preset değerleriyle senkron kalması sağlandı.
  - `ConfigManager` varsayılan yolu `/var/lib/victus-max-daemon/fan_config.json` olarak güncellendi ve geriye dönük fallback korundu.
  - `monitoring.rs` içerisindeki eski `omen-space-daemon` restart komutları `victus-max-daemon` olarak düzeltildi.



---

## [2026-09-29] Fix | Systemd Mount Namespacing Hatası (226/NAMESPACE) ve Konfigürasyon Yolları Geçişi

- **İşlem Türü:** Sistem Servisi & Güvenlik Sandbox Düzeltmesi (Systematic Debugging)
- **Kapsam:** `data/victus-max-daemon.service`, `setup.sh`, `src/victus-max-daemon/src/mux.rs`, `src/victus-max-daemon/src/platform.rs`, `src/victus-max-daemon/src/power.rs`, `src/victus-max-daemon/src/rgb/mod.rs`, `src/victus-max-daemon/src/ryzen.rs`, `src/victus-max-daemon/src/undervolt.rs`, `src/victus-max-daemon/src/hid_wizard.rs`.
- **Kök Neden:**
  - `data/victus-max-daemon.service` dosyasında `ReadWritePaths=` içinde `/etc/omen-space` yolu bulunuyordu ancak dosya sisteminde bu dizin bulunmadığı için systemd mount namespacing kurulumu `status=226/NAMESPACE` hatasıyla çöküyor ve servis başlatılamıyordu.
- **Detaylar:**
  - `data/victus-max-daemon.service` dosyasına `ConfigurationDirectory=victus-max` eklendi; `/etc/victus-max` dizini systemd tarafından otomatik oluşturulup yetkilendirildi.
  - `ReadWritePaths=` altındaki tüm opsiyonel yollar (`-/etc/victus-max`, `-/etc/omen-space`, `-/var/lib/victus-max-daemon`, `-/var/lib/victus-max`, `-/var/lib/omen-space-daemon`, `-/etc/udev/hwdb.d`) systemd standardına uygun olarak `-` önekiyle tanımlandı (böylece mevcut olmayan yollar namespacing çökmesine yol açmayacak şekilde yoksayılıyor).
  - Daemon altındaki tüm mikroservislerin (`mux`, `platform`, `power`, `rgb`, `ryzen`, `undervolt`, `hid_wizard`) birincil yapılandırma dosya yolları `/etc/victus-max/` altına taşındı; mevcut kullanıcı ayarları için eski yoldan okuma (fallback) korundu.
  - `setup.sh` dosyasına `/etc/victus-max/keymaps` ve `/var/lib/victus-max-daemon` dizinlerinin kurulum anında oluşturulması eklendi.
  - Workspace testleri (52/52) ve release derlemesi başarıyla tamamlandı.


---

## [2026-09-29] Fix | Fan Curve Editor Pencere Saydamlık Hatası ve Libadwaita Dönüşümü

- **İşlem Türü:** Arayüz & Görsel Hata Düzeltmesi (UI/UX Bugfix)
- **Kapsam:** `src/victus-max-gui/src/fan_curve_editor.rs`, `src/victus-max-gui/src/performance_control.rs`.
- **Kök Neden:**
  - Fan Curve Editor penceresi çıplak bir `gtk::Window` üzerine `.css_classes(["os-card"])` verilerek açılıyordu. `@card_bg_color` Libadwaita karanlık modunda saydamlık (alfa) içerdiği için ve pencere seviyesinde opak arka plan render edilmediği için tüm pencere hayalet gibi saydam kalıyordu.
  - Buna ek olarak `DrawingArea` içerisinde `cr.set_operator(gtk::cairo::Operator::Clear); cr.paint()` çağrısı yapıldığı için Cairo, pencerenin alfa tamponunu `0.0`a çekerek grafik alanını doğrudan masaüstüne / altındaki ana pencereye saydam delik açıyordu.
  - Client-side decoration (HeaderBar) bulunmadığı için KDE KWin sunucu taraflı yabancı başlık çubuğu çiziyordu.
- **Detaylar:**
  - `fan_curve_editor.rs` içerisinde pencere `adw::Window` + `adw::ToolbarView` + `adw::HeaderBar` yapısına dönüştürüldü; böylece pencere opak Adwaita zeminine (`@window_bg_color`), yerel Adwaita başlığına ve kapatma butonuna kavuştu.
  - Grafik çizim alanı `os-card` kart konteyneri içine alındı ve `cairo::Operator::Clear` kaldırılarak `Operator::Over` ile koyu kontrastlı şık bir grafik arka planı (`rgba(0,0,0,0.25)`) çizildi.
  - Aynı `Operator::Clear` temizliği `performance_control.rs` üzerindeki ana fan eğrisi çizicisine de uygulandı.

---

## [2026-09-29] Cleanup | images/omenspace.png Dosyasının Silinmesi

- **İşlem Türü:** Görsel Varlık & Temizlik (Asset Cleanup)
- **Kapsam:** `images/omenspace.png`.
- **Detaylar:**
  - `README.md` ve arayüz artık `images/victus-max.png` kullandığı için atıl ve mükerrer kalan eski logo dosyası `images/omenspace.png` depodan kaldırıldı.

---

## [2026-09-29] Feature | GitHub Entegrasyonlu OTA Güncelleme ve Sürüm Mimarisi (ADR-006)

- **İşlem Türü:** Özellik Ekleme, Güvenlik Politikası ve Mimari Dokümantasyon
- **Kapsam:** `src/victus-max-gui/`, `scripts/`, `data/`, `.github/workflows/`, `setup.sh`, `docs/victus-max-wiki/`, `docs/gui.md`.
- **Detaylar:**
  - GUI Güncelleme Merkezinin GitHub REST API entegrasyonu tamamlandı; Canary (Git main) ve Stable (GitHub Releases) kanalları oluşturuldu.
  - Kullanıcının kanal tercihi `~/.config/victus-max/settings.json` içerisine kalıcı konfigürasyon olarak bağlandı; tekrarlayan seçim zorunluluğu ortadan kaldırıldı.
  - GitHub üzerinde henüz Release bulunmadığında 404 hatasını önleyen ve kullanıcıyı bilgilendirerek en son geliştirme commit'lerini gösteren sıfır-404 otomatik Canary fallback mekanizması eklendi.
  - Root yetkisiyle çalışan bağımsız `/usr/libexec/victus-max/victus-max-updater` betiği ve `org.hp.victusmax.update` Polkit eylem politikası entegre edildi.
  - GUI üzerinde gerçek zamanlı aşama takip çubuğu (`[STAGE:...]`) ve güncelleme sonrası tek tıkla yeniden başlatma butonu geliştirildi.
  - GitHub Actions etiket tetiklemeli otomatik release iş akışı (`release.yml`) ve yerel sürüm yükseltme aracı (`scripts/release.sh`) hazırlandı; `setup.sh` önceden derlenmiş ikili paketleri algılayacak şekilde güncellendi.
- **Mimari Karar:** [[adr-006-github-update-and-release-architecture]]

---

## [2026-09-30] Cleanup | Komut Satırı Arayüzü (CLI) İsimlendirme ve Victus ASCII Logo Güncellemesi

- **İşlem Türü:** CLI Marka Temizliği ve Donanım Tespiti (De-branding & UI Polish)
- **Kapsam:** `src/victus-max-cli/src/fetch.rs`, `src/victus-max-cli/src/main.rs`, `src/victus-max-cli/src/commands/overlay.rs`, `docs/victus-max-wiki/user-experience/command-line-interface.md`, `docs/architecture.md`, `docs/cli.md`.
- **Detaylar:**
  - `victus-max-cli` interaktif terminal panosu (`fetch.rs`) içerisindeki `omen-cli >` istemci metni `victus-max-cli >` olarak güncellendi.
  - Alt komut çalıştırıcı fallback dizesindeki `omen-cli` çağrısı `victus-max-cli` olarak düzeltildi.
  - Canlı pano ve fetch çıktısı için HP Victus modellerine özel geometrik mavi-mor degradeli Victus "V" ASCII logosu eklendi; OMEN modellerinde OMEN eşkenar dörtgeni, Victus modellerinde ise Victus logosu dinamik olarak gösterilecek şekilde yapılandırıldı.
  - `overlay.rs` ve `main.rs` içerisindeki eski marka referansları arındırıldı; ilgili dokümantasyon sayfaları güncellendi.

---

## [2026-09-30] Feature | Resmi Victus Max Amblemi ASCII Sanatı (CLI Emblemi)

- **İşlem Türü:** CLI Görsel Tasarım & Terminal Estetiği (ASCII Art Polish)
- **Kapsam:** `src/victus-max-cli/src/fetch.rs`.
- **Detaylar:**
  - `images/victus-max.png` resmi ambleminden esinlenerek 12 satırlık, 26 sütunluk simetrik Victus Max amblemi ASCII sanatı tasarlandı.
  - OMEN dış eşkenar dörtgeni, tepe tribal alev boynuzları, kılıç omurgası ve alt yarıda kalın, parlak Victus "V" kanatları (`##\ ... /##`) terminal karakterleriyle stilize edildi.
  - RGB ANSI Truecolor (24-bit) paleti ile amblemin kızıl metalik çerçevesi, alevli turuncu boynuzları ve kor halindeki altın sarısı Victus V geçişi (`#FF2020` -> `#FF8000` -> `#FFD020`) renklendirildi.
  - Canlı pano ve `victus-max-cli fetch` sistem bilgi satırlarıyla (12 satır) birebir piksel hizalaması sağlandı.

---

## [2026-09-30] Feature | Yüksek Çözünürlüklü Unicode Half-Block Victus Max Amblemi

- **İşlem Türü:** Terminal Grafiği & Detaylı Piksel Sanatı (Unicode Half-Block TrueColor)
- **Kapsam:** `src/victus-max-cli/src/fetch.rs`.
- **Detaylar:**
  - Karakter alanı sınırını aşmak ve amblemdeki tüm ince detayları (tribal alev boynuzları, kafatası/maske göz yuvaları, kaş kavisleri, kılıç kabzası ve altın Victus V) terminalde net gösterebilmek amacıyla `▀` (`\u2580`) Unicode yarım-blok (half-block) tekniğine geçildi.
  - Her karakter hücresinin üst pikseli ön plan (`38;2;R;G;B`), alt pikseli arka plan (`48;2;R;G;B`) olarak bağımsız renklendirilerek 28×14 karakterlik alanda tam **28×28 (784 piksel) 24-bit TrueColor** çözünürlük elde edildi.
  - `get_ascii_logo` fonksiyonu ortaklaştırılarak kod tekrarı önlendi; OMEN ve Victus modelleri için dinamik donanım tespiti korundu.

---

## [2026-09-30] Feature | Dinamik ve Adaptif Logo Çözünürlüğü (Compact / Medium / Ultra HD)

- **İşlem Türü:** Terminal Boyut Tespiti & Dinamik Grafik Ölçekleme (Adaptive Multi-Resolution)
- **Kapsam:** `src/victus-max-cli/src/fetch.rs`.
- **Detaylar:**
  - `crossterm::terminal::size()` ile terminal penceresinin satır ve sütun boyutunu anlık analiz eden `detect_logo_resolution` fonksiyonu eklendi.
  - 3 farklı çözünürlük seviyesi entegre edildi:
    - **Ultra HD (40×20 karakter, 40×40 = 1.600 TrueColor piksel):** Geniş veya tam ekran pencerelerde (cols >= 105, rows >= 25) devreye girerek maksimum detay sağlar.
    - **Medium (34×17 karakter, 34×34 = 1.156 TrueColor piksel):** Standart orta boy terminallerde (cols >= 88, rows >= 20) çalışır.
    - **Compact (28×14 karakter, 28×28 = 784 TrueColor piksel):** Küçük veya bölünmüş (tiled) pencerelerde metin kaymasını önleyecek şekilde kompakt gösterim sunar.
  - Canlı interaktif panoda (`run_live_dashboard`) `Event::Resize` dinleyicisi eklenerek pencere boyutu değiştirildiğinde logonun anında ve sıfır gecikmeyle yeni boyuta adapte olması sağlandı.
  - `fetch` çıktısında ve canlı panoda `logo_width` üzerinden dinamik padding uygulanarak telemetri metin hizalaması pürüzsüzleştirildi.

---

## [2026-09-30] Fix | CLI Canlı Panosunda Satır Kayması ve Sol Kenar Karakter Artığı Temizliği

- **İşlem Türü:** Hata Düzeltme & Terminal Çizim Hijyeni (ANSI Line-Wrap & Margin Hygiene)
- **Kapsam:** `src/victus-max-cli/src/fetch.rs`.
- **Detaylar:**
  - Canlı panoda (`victus-max-cli`) terminalin sol kenarında beliren hayalet `6_` ve `r_` karakter artıkları araştırıldı.
  - Sorunun kök nedeni tespit edildi: `Host:` satırındaki `16-s0xxx` model adının ve `Thermal/Fans:` satırındaki `[better-auto]` etiketinin 90-100 sütunluk terminallerde sağ kenarı aşarak alt satırın 0. sütununa taşması (`auto-wrap`) ve eski kodun çizime `(2, row_idx)` koordinatından başlayıp 0. sütunu silmemesi nedeniyle bu karakterlerin ekranda donup kalması.
  - `crossterm::terminal::DisableLineWrap` ile canlı pano süresince otomatik satır taşması engellendi; çıkışta `EnableLineWrap` ile terminal eski haline döndürüldü.
  - Çizim imleci satırın en başına (`MoveTo(0, row_idx)`) çekilerek her satır başına `\x1b[2K` (satırın tamamını temizle) ANSI dizisi eklendi.
  - Çözünürlük geçiş eşikleri güvenlik payı verilerek güncellendi (Medium $\ge 100$, Large $\ge 115$).
  - `Thermal/Fans` satırı (`Fans: 2600 RPM` veya `Fans: 2400/2600 RPM`) kompaktlaştırılarak gereksiz etiket tekrarları kaldırıldı ve satır genişliği 30 karakter daraltıldı.
  - Pencere yeniden boyutlandığında (`Event::Resize`) ekran `Clear(ClearType::All)` ile sıfırlanarak küçülme anındaki hayalet karakterler tamamen yok edildi.

---

## [2026-09-30] Fix | GUI Güncelleyici Polkit Kimlik Doğrulama Dayanıklılığı ve Root Yönetimi

- **İşlem Türü:** Hata Düzeltme & Yetki Yükseltme Dayanıklılığı (Polkit Resilience & Privilege Handling)
- **Kapsam:** `src/victus-max-gui/src/updater.rs`, `src/victus-max-gui/src/i18n.rs`, `setup.sh`.
- **Detaylar:**
  - Grafiksel güncelleme esnasında `pkexec` üzerinden çağrılan güncelleyicinin, oturumda aktif bir Polkit Authentication Agent bulunmadığında veya GUI `sudo` ile çalıştırıldığında `/dev/tty` açılamaması sonucu `exit status: 127` ile başarısız olması sorunu çözüldü.
  - `unsafe { libc::geteuid() == 0 }` kontrolü eklenerek GUI zaten root/sudo ile çalıştırıldıysa gereksiz `pkexec` çağrısı baypas edildi ve doğrudan güncelleyici ikili dosyası tetiklendi.
  - `is_polkit_auth_error` fonksiyonu ile Polkit kimlik doğrulama ajanı eksikliği (`exit status: 127`, `textual authentication agent`, `No such device or address`, `not authorized`) anında yakalanıp kullanıcıya Türkçe ve İngilizce net rehberlik metni (`polkit_agent_missing`) sağlandı.
  - Hata durumunda terminal çıktı penceresi otomatik olarak açılarak (`term_toggle.set_active(true)`) kullanıcının doğrudan teknik detayları görmesi sağlandı.
  - `resolve_updater_bin` fonksiyonu `/usr/bin/victus-max-updater` ve yerel çalışma dizinlerini de denetleyecek biçimde güçlendirildi.
  - `setup.sh` içerisine `/usr/bin/victus-max-updater` sembolik bağı eklenerek güncelleyicinin doğrudan terminal komutu olarak da çalıştırılabilmesi sağlandı.

---

## [2026-10-10] Feature | İşlemciye Duyarlı (Adaptive) AMD Ryzen Güç Limitleri ve Canlı PMTable Telemetrisi

- **İşlem Türü:** Mimari Genişletme, Canlı Donanım Okuma ve Adaptif GTK4 Arayüzü
- **Kapsam:** `src/victus-max-daemon/src/cpu_tier.rs`, `src/victus-max-daemon/src/power.rs`, `src/victus-max-daemon/src/undervolt.rs`, `src/victus-max-gui/src/daemon_client.rs`, `src/victus-max-gui/src/undervolt.rs`, `src/victus-max-gui/src/i18n.rs`, `docs/victus-max-wiki/`.
- **Detaylar:**
  - `cpu_tier.rs` modülü oluşturularak işlemciler mimari ve TDP sınıflarına (`AmdUltraLow` U-serisi, `AmdStandardHs` HS/H-serisi, `AmdExtremeHx` HX-serisi, `IntelStandard`, `IntelExtreme`) ayrıldı ve her sınıf için güvenli STAPM/PL1, Fast PPT/PL2 ve Tctl sıcaklık aralıkları tanımlandı.
  - Systemd kısıtlı `PATH` ortamlarında `/usr/local/bin/ryzenadj` ikili dosyasının her zaman bulunabilmesi için `resolve_ryzenadj_bin` çözümleyicisi eklendi.
  - `power.rs` içindeki `GetPowerProfile` metodu, AMD Ryzen işlemcilerde `ryzenadj -i` çıktısını 500ms zaman aşımıyla sorgulayıp `STAPM LIMIT`, `PPT LIMIT FAST`, `PPT LIMIT SLOW` ve `THM LIMIT CORE` değerlerini canlı olarak ayrıştıracak (`parse_ryzenadj_info_output`) ve D-Bus JSON yanıtında sunacak şekilde genişletildi; ayrıca 3 limiti bağımsız yazmak için `SetRyzenPowerLimits(enabled, stapm, slow, fast)` D-Bus metodu eklendi.
  - `undervolt.rs` (GUI) içerisindeki Power Limits grubu AMD Ryzen algılandığında başlıklarını, alt açıklamalarını, slider aralıklarını ve canlı değerlerini otomatik olarak `STAPM`, `PPT Slow`, `PPT Fast` (3 bağımsız güç slider'ı) ve doğrudan `Tctl Sıcaklık Limiti (°C)` olarak güncelleyecek biçimde uyarlandı.
  - Türkçe ve İngilizce dil sözlüklerine (`i18n.rs`) AMD Ryzen güç ve sıcaklık terimleri eklendi.
- **Mimari Karar:** [[adr-007-adaptive-ryzen-power-limits]]
- **Test ve Doğrulama:** `cpu_tier_test` (5 test) ve `ryzen_telemetry_test` (2 test) dahil tüm çalışma alanı testleri (`cargo test --workspace`, toplam 69 test) sıfır hatayla geçti.

---

## [2026-10-10] Fix | AMD Ryzen SMU Posta Kutusu Çakışması, Eski Config Göçü ve Canlı Limit Senkronizasyonu

- **İşlem Türü:** Kök Neden Analizi & Eşzamanlılık Düzeltmesi (Systematic Debugging)
- **Kapsam:** `src/victus-max-daemon/src/cpu_tier.rs`, `src/victus-max-daemon/src/power.rs`, `src/victus-max-daemon/src/undervolt.rs`, `src/victus-max-types/src/lib.rs`, `src/victus-max-gui/src/daemon_client.rs`, `src/victus-max-gui/src/undervolt.rs`, `docs/victus-max-wiki/`.
- **Kök Neden:**
  1. `performance_control.rs` (1.5s zamanlayıcı) ve `victus-max-tray` (3.0s zamanlayıcı) aktif güç profilini öğrenmek için sürekli `GetPowerProfile()` metodunu çağırıyordu. `ryzenadj -i` çağrısı doğrudan `GetPowerProfile()` içinde yer aldığı için dakikada ~40 kez eşzamanlı `ryzenadj -i` süreci başlatılıyor ve `/dev/mem` üzerinden PCI SMN `0x60`/`0x64` yazmaçlarında yarış koşulu (race condition) oluşarak MP1 SMU posta kutusunu kilitliyordu (`Unable to get MP1 SMU Obj`).
  2. `/etc/victus-max/power.json` içinde Ryzen desteği öncesinden kalma Intel varsayılanları (`pl1_w: 45, pl2_w: 80, pl_enabled: true, slow_ppt_w: 0`) ve `/etc/victus-max/undervolt.json` içinde `tcc_offset: 11` (`89°C`) bulunduğu için servis başlangıcında ve SMU kilitlenmesi sonrası fallback durumunda arayüzde gerçek donanım değerleri (`48W / 43W / 48W / 88°C`) yerine `75W / 45W / 80W / 89°C` görünüyordu.
  3. Gelişmiş Güç (`undervolt.rs`) sayfası yalnızca uygulama ilk açıldığında bir kez yükleniyor, sekmeye geçiş yapıldığında (`connect_map`) güncellenmiyordu.
- **Detaylar:**
  - `GetPowerProfile()` içerisinden `ryzenadj -i` çağrısı tamamen çıkarıldı; canlı limit sorgusu yalnızca Gelişmiş Güç sekmesi tarafından çağrılan yeni `GetPowerLimitsInfo()` D-Bus metoduna ayrıldı.
  - Tüm `ryzenadj` çağrıları `cpu_tier::run_ryzenadj` altında `RYZENADJ_MUTEX` (`tokio::sync::Mutex`) ve 40ms SMU dinlenme aralığı ile serileştirildi; `fetch_live_ryzen_metrics` için 2 saniyelik önbellek eklendi.
  - AMD sistemlerde `slow_ppt_w == 0` olan eski `power.json` kayıtları başlangıçta otomatik temizlenerek (`pl_enabled = false`) donanım limitlerini ezmesi önlendi.
  - GUI tarafında sekme her görüntülendiğinde (`page.connect_map`) canlı değerlerin yenilenmesi ve `apply_all_tuning_sync` ile ayarların tek görevde sıralı uygulanması sağlandı.
- **Mimari Karar:** [[adr-007-adaptive-ryzen-power-limits]]

---

## [2026-10-11] Feature | AMD Ryzen Nesillerine Göre STAPM–Fast PPT SMU Bağlaması ve Hiyerarşi Koruması

- **İşlem Türü:** Donanım SMU Modellemesi, Hiyerarşik Doğrulama ve GTK4 Reaktif Slider Senkronizasyonu
- **Kapsam:** `src/victus-max-daemon/src/cpu_tier.rs`, `src/victus-max-daemon/src/power.rs`, `src/victus-max-daemon/tests/cpu_tier_test.rs`, `src/victus-max-gui/src/daemon_client.rs`, `src/victus-max-gui/src/undervolt.rs`, `docs/victus-max-wiki/`.
- **Detaylar:**
  - `is_zen4_or_newer_ryzen` fonksiyonu ile AMD Ryzen işlemciler mimari nesline göre ayrıştırıldı: Phoenix, Hawk Point, Dragon Range (Zen 4) ve Strix Point (Zen 5) işlemcilerde `stapm_fast_linked = true` bayrağı tanımlandı; Rembrandt (Zen 3+), Cezanne/Lucienne (Zen 3/2) işlemcilerde `stapm_fast_linked = false` olarak belirlendi.
  - SMU donanım kısıtları modellendi: Zen 4 ve Zen 5'te `--fast-limit` komutunun STAPM sınırını otomatik olarak Fast PPT'ye kilitlemesi (`STAPM = PPT Fast`) ve `PPT Slow <= PPT Fast` gereksinimi; eski nesillerde ise 3 bağımsız kademeli `STAPM <= PPT Slow <= PPT Fast` hiyerarşisi doğrulandı.
  - `cpu_tier::normalize_ryzen_limits` fonksiyonu geliştirildi; D-Bus üzerinden gelen güç limitleri donanıma yazılmadan önce CPU nesline göre normalize edilerek geçersiz SMU parametrelerinin önüne geçildi.
  - `GetPowerLimitsInfo` D-Bus metodu ve JSON şeması `stapm_fast_linked` alanını dönecek şekilde genişletildi.
  - GTK4 Libadwaita Gelişmiş Güç arayüzünde (`undervolt.rs`) ilişkisel slider senkronizasyonu uygulandı: Zen 4/5 sistemlerde STAPM ve PPT Fast slider'ları çift yönlü reaktif olarak bağlandı (`STAPM <-> PPT Fast`), PPT Slow ise tavanı aşmayacak şekilde sınırlandı. Eski Ryzen nesillerinde ise 3 slider bağımsız kalırken `STAPM <= PPT Slow <= PPT Fast` hiyerarşisi korundu.
- **Mimari Karar:** [[adr-007-adaptive-ryzen-power-limits]]
- **Test ve Doğrulama:** `test_zen4_and_zen5_stapm_fast_linked_detection`, `test_normalize_ryzen_limits_coupled_vs_hierarchy` ve GTK4 slider etkileşim simülasyonu testleri dahil tüm workspace testleri (`cargo test --workspace`) başarıyla geçti.

---

## [2026-10-12] Fix | Polkit Güncelleme Yetkilendirmesi (`/dev/tty` Exit 127) ve GUI Süreç Koruma

- **İşlem Türü:** Polkit Yetkilendirme Politikası, Güncelleyici Betik Dayanıklılığı ve CI/CD İyileştirmesi
- **Kapsam:** `data/org.hp.victusmax.update.policy`, `data/org.hp.victusmax.rules`, `scripts/victus-max-updater.sh`, `setup.sh`, `.github/workflows/release.yml`, `docs/victus-max-wiki/platform-integration/polkit-dbus-security.md`.
- **Detaylar:**
  - Güncelleme Merkezi'nde `pkexec /usr/libexec/victus-max/victus-max-updater` çağrıldığında, oturumda grafiksel bir Polkit ajanı bulunmaması veya uygulamanın `setup.sh` sonrası `su -` alt oturumundan başlatılması nedeniyle `pkexec`'in metin tabanlı `/dev/tty` ajanına düşüp `exit status: 127` (`No such device or address`) hatası vermesi çözüldü.
  - `data/org.hp.victusmax.update.policy` içinde yalnızca `/usr/libexec/victus-max/victus-max-updater` ikilisine bağlı olan `org.hp.victusmax.update` eylemi `yes` olarak güncellendi ve `/usr/share/polkit-1/rules.d/org.hp.victusmax.rules` kuralı eklendi.
  - `scripts/victus-max-updater.sh` ve `setup.sh` içine `PKEXEC_UID` üzerinden `SUDO_USER` çözümlemesi eklendi; böylece `pkexec` altında çalıştığında kullanıcının yerel Rust araç zincirinin (`~/.cargo/bin/cargo`) doğrudan bulunması sağlandı.
  - `VICTUS_MAX_GUI_UPDATE=1` ortam değişkeni ile GUI içinden başlatılan güncellemelerde `setup.sh` içerisindeki `killall victus-max` adımı atlanarak güncelleme sırasında GUI penceresinin kapanması ve çıktı borusunun (`SIGPIPE`) kopması engellendi.
- **İlgili Bağlantılar:** [[polkit-dbus-security]], [[adr-006-github-update-and-release-architecture]]

---

## [2026-10-13] Feature | Güç Modu Profilleri (Presets), 30sn Emniyet Kilidi ve Donanımsal Koruma Mimarisi

- **İşlem Türü:** Güç Yönetimi, Donanımsal Limit Koruma (Watchdog), Olay Eşitleme ve GTK4 Arayüz Genişletmesi
- **Kapsam:** `src/victus-max-types/`, `src/victus-max-daemon/`, `src/victus-max-gui/`, `src/victus-max-cli/`, `docs/victus-max-wiki/`.
- **Detaylar:**
  - `omen_types` içine `PowerModeLimits` ve `PowerModeProfilesConfig` yapıları ile `PowerProxy` D-Bus genişletmeleri eklendi.
  - `cpu_tier.rs` modülüne CPU sınıfına (AMD Standard HS, Extreme HX, UltraLow U, Intel Standard H/P, Intel Extreme HX) göre optimize edilmiş varsayılan limit profilleri (`build_default_power_presets`) eklendi.
  - `PowerService` (`power.rs`) içine `/etc/victus-max/power.json` kalıcılığı ile `limit_presets`, `manual_override`, `apply_preset_limits`, `get_power_presets`, `set_power_preset_mode`, `save_power_preset`, `reset_power_presets` D-Bus metotları eklendi.
  - 30 saniyelik arka plan sessiz emniyet kilidi (watchdog) tokio zamanlayıcısı eklendi; HP EC/BIOS'un limitleri sıfırlaması sessizce engellendi.
  - `power_automation.rs` içinde PPD `ActiveProfile` sinyalleri, AC şarj takma/çıkarma döngüsü ve `org.freedesktop.login1` uykudan uyanma (`PrepareForSleep(false)`) sinyali dinlenerek güç profilleri ve limitleri otomatik senkronize edildi.
  - GTK4 Libadwaita Gelişmiş Güç (`undervolt.rs`) sayfasına 3 adet güç modu kartı (Power Save, Balanced, Performance), durum rozeti, çift yönlü slider override senkronizasyonu ve mimariye duyarlı (Ryzen: STAPM/Slow/Fast/Temp; Intel: PL1/PL2/TCC) modal düzenleme penceresi eklendi.
  - `victus-max-cli power presets [list|set|reset]` ve `victus-max-cli power limits` komutları eklenerek terminalden canlı durum sorgulama ve profil yönetimi sağlandı.
- **Mimari Karar:** [[adr-008-power-profile-limit-presets]]
- **Test ve Doğrulama:** Tüm çalışma alanı testleri (`cargo test --workspace`, 118 test) sıfır hatayla geçti.

---

## [2026-10-14] Fix | Gelişmiş Güç Sekmesinde Dış Profil Değişimlerinin Canlı (Reaktif) Senkronizasyonu

- **İşlem Türü:** Hata Düzeltme & GUI Canlı Durum Eşitleme
- **Kapsam:** `src/victus-max-gui/src/undervolt.rs`.
- **Detaylar:**
  - Kullanıcı Gelişmiş Güç (`undervolt.rs`) sekmesindeyken sistem tepsisinden (Tray), PPD'den veya kısayol tuşlarından performans modunu değiştirdiğinde, aktif preset kartının ve durum rozetinin sadece sekme değiştirip geri dönüldüğünde güncellenmesi kozmetik sorunu giderildi.
  - Sayfa görünürken (`page.is_mapped()`) her 1000ms'de çalışan asenkron hafif kontrol döngüsü (`glib::timeout_add_local`) eklendi; `get_power_presets_async()` çağrılarak daemon durumu ile UI durumu arasındaki farklar tespit edildiğinde aktif kart seçimi, durum rozeti ve slider değerleri canlı olarak güncellendi.
  - Kullanıcı slider'ları manuel olarak hareket ettirirken veya preset düzenleme penceresindeyken canlı döngünün kullanıcı girdisini ezmesini engelleyen güvenlik kilidi eklendi.
  - `set_active_card` içinde butonların mevcut aktiflik durumları kontrol edilerek gereksiz GTK sinyal tetiklemeleri ve yeniden çizimler önlendi.
- **İlgili Bağlantılar:** [[adr-008-power-profile-limit-presets]], [[power-service]]
- **Test ve Doğrulama:** `test_external_preset_change_detection` birim testi eklendi; `cargo test --workspace` ile 119 testin tamamı başarıyla geçti.


