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

