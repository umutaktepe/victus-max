# Victus Max MOC — Yaşayan Mimari İçerik Haritası (Index)

Bu dizin, Victus Max kod tabanının **Andrej Karpathy LLM Wiki / Living Architecture** paradigmasına göre modellenmiş ana İçerik Haritasıdır (Map of Content - MOC). Sistem hiyerarşisi sayısal kitap bölümleriyle değil; fonksiyonel mühendislik rolleri, mimari kararlar ve atomik ilişkisel kümeler (clusters) ile yapılandırılmıştır.

---

## 🏛️ Mimari Karar Kayıtları (Decisions / ADR)
Sistem mimarisindeki temel teknik seçimler, gerekçeleri, değerlendirilen alternatifler ve ödünleşimler (trade-offs):

- [[adr-001-rust-daemon-client-split]] — Root yetkili Rust arka plan servisi ile kullanıcı alanı istemcilerinin D-Bus üzerinden ayrılması ve ayrıcalık izolasyonu.
- [[adr-002-wmi-vs-direct-ec-arbitration]] — Güvensiz anakartlarda doğrudan EC yazımlarını engelleyen ve güvenli modellerde hwmon fallback sağlayan hibrit tahkim mekanizması.
- [[adr-003-companion-dkms-driver-non-clashing]] — Stock `hp-wmi` çekirdek sürücüsüyle çakışmadan WMI GUID paylaşımı yapan yoldaş DKMS RGB modülü mimarisi.
- [[adr-004-native-msr-and-smu-mailbox-tuning]] — Dış CLI araçlarına bağımlı kalmadan Intel MSR ve AMD SMU posta kutusu protokollerinin yerel Rust ile işletilmesi.
- [[adr-005-better-auto-proactive-fan-and-victus-max]] — Proaktif Better Auto çift matrisli fan algoritması, 10s EC yazma aralığı, ayarlanabilir taban/tavan ve bağımsız Victus Max mimarisi.

---

## ⚙️ Donanım ve Çekirdek Arabirimleri (Hardware Drivers)
Linux çekirdeği, ACPI, Gömülü Denetleyici (EC), MSR ve SMU seviyesinde donanımla temas kuran çekirdek arabirimler:

- [[hp-wmi-driver]] — HP WMI ACPI arabirimi, donanımsal termal profil anahtarlama ve GPU MUX sysfs düğümü.
- [[hp-omen-extra-dkms]] — Çekirdek içi WMI sorgularıyla klavye arka aydınlatmasını yöneten ve `/sys/devices/platform/hp-omen-extra/` düğümlerini açan DKMS sürücüsü.
- [[embedded-controller-ec]] — `/sys/kernel/debug/ec/ec0/io` portu, fan görev döngüsü yazmaçları, güvenlik duvaları ve Caps Lock panik engelleme mantığı.
- [[cpu-msr-undervolt]] — Intel MSR `0x150` ve `0x1A2` yazmaçları, 5 bağımsız voltaj düzlemi ve bit paketleme matematiği.
- [[amd-ryzen-smu]] — Zen1+'dan Zen5'e kadar AMD işlemcilerin tespiti, SMU posta kutusu komutları, STAPM/PPT limitleri ve Curve Optimizer.

---

## 🧠 Sistem Katmanı ve Çekirdek Servisler (System Daemon)
Root yetkileriyle arka planda çalışan `victus-max-daemon` mikroservisleri ve koruma mekanizmaları:

- [[daemon-overview]] — Daemon mimarisi, Tokio asenkron çalışma zamanı, D-Bus nesne sunucusu ve başlatma sırası.
- [[fan-service]] — Fan hız modları, Better Auto proaktif algoritması, özel spline eğrileri, keep-alive döngüsü ve 95°C acil durum termal koruma devresi.
- [[power-service]] — ACPI termal profilleri (`power-saver`, `balanced`, `performance`), Intel RAPL PL1/PL2 tavanları ve GPU TGP sınırlama.
- [[rgb-service]] — 4-Zone, Per-Key ve Lightbar donanımları için statik renkler, dinamik LED animasyonları ve eşleme motoru.
- [[mux-service]] — dGPU ve iGPU ekran paneli yönlendirmesi, WMI anahtarlama ve yeniden başlatma gereksinim koordinasyonu.
- [[undervolt-service]] — Intel MSR undervolting servisi, canlı yazmaç doğrulama ve yazılımsal güvenlik sınırları.
- [[ryzen-service]] — AMD Ryzen işlemciler için güç limitleri, sıcaklık hedefleri ve kalıcı SMU optimizasyonu.
- [[sysmon-service]] — Hwmon, NVML ve sysfs üzerinden telemetri derleyen, periyodik D-Bus sinyali yayan izleme servisi.
- [[conflict-detector]] — NBFC, Throttled, Ryzenadj ve OGHAAgent gibi rakip donanım araçlarıyla yarış durumlarını engelleyen dedektör.
- [[game-automation-service]] — Çalışan oyunları ve 3D süreçleri izleyerek otomatik performans profili uygulayan servis.
- [[power-automation-service]] — GNOME PowerProfilesDaemon (PPD) sinyal senkronizasyonu ve AC/Pil geçiş izleyicisi.
- [[hotkey-monitor]] — `/dev/input/event*` üzerinden Shift+F2 ve OMEN tuşunu sıfır gecikmeyle yakalayan evdev dinleyicisi.
- [[platform-service]] — Pil sağlığı bakım modu (%80 şarj tavanı), fan tozu temizleme ve hızlı teşhis arşivi servisi.

---

## 📐 Veri Şemaları ve Protokoller (Interfaces)
Sistem bileşenleri arasındaki IPC sözleşmeleri, D-Bus arayüz tanımları ve JSON veri formatları:

- [[dbus-ipc-protocol]] — `org.hp.omen` D-Bus ad alanı, zbus proxy trait mimarisi ve sinyal akış şeması.
- [[fan-dbus-interface]] — `org.hp.omen.Fan` metotları, sinyalleri ve özel fan eğrisi JSON veri formatı.
- [[power-dbus-interface]] — `org.hp.omen.Power` arayüzü ve `/etc/victus-max/power.json` dosya şeması.
- [[rgb-dbus-interface]] — `org.hp.omen.Rgb` metotları, Hex renk kodlaması ve interaktif eşleme komutları.
- [[mux-dbus-interface]] — `org.hp.omen.Mux` sözleşmesi ve ekran yönlendirme durum yanıtları.
- [[undervolt-dbus-interface]] — `org.hp.omen.Undervolt` MSR ofset arayüzü ve yanıt modelleri.
- [[ryzen-dbus-interface]] — `org.hp.omen.Ryzen` SMU ayar arayüzü ve `/etc/victus-max/ryzen.json` şeması.
- [[sysmon-dbus-interface]] — `org.hp.omen.SysMon` telemetri yayını ve tanı raporu sözleşmesi.
- [[system-telemetry-spec]] — `SystemStats` dinamik sensör veri paketi ve `HardwareSpecs` statik donanım modeli.
- [[board-capabilities-matrix]] — `boards.json` donanım kabiliyet matrisi, `LinuxCapabilityClass` ve fallback mantığı.
- [[app-profiles-spec]] — `/etc/victus-max/app_profiles.json` otomatik oyun kural şeması.

---

## 🖥️ İstemciler ve Kullanıcı Deneyimi (User Experience)
Kullanıcı alanında (unprivileged) koşan grafik, komut satırı ve panel arayüzleri:

- [[gui-application]] — GTK4 ve Libadwaita ile inşa edilmiş ana masaüstü kontrol merkezi, sekme mimarisi ve D-Bus istemcisi.
- [[quick-hud-overlay]] — Oyun esnasında Shift+F2 ile ekrana gelen, sıfır gecikmeli yarı saydam Wayland GTK4 HUD paneli.
- [[system-tray]] — Masaüstü bildirim alanında çalışan hafif (<2MB) hızlı profil seçici ve gösterge simgesi.
- [[command-line-interface]] — Terminal ve otomasyon betikleri için `victus-max-cli` (ve `omen-cli`) komut hiyerarşisi ve ASCII donanım bilgi çıktısı (`fetch`).
- [[fan-curve-editor-ui]] — GTK4 DrawingArea üzerinde çalışan etkileşimli fan eğrisi ve spline çizim bileşeni.

---

## 🔒 Sistem Entegrasyonu ve Güvenlik (Platform Integration)
Sistem seviyesinde kurulum, güvenlik sınırları, izin kuralları ve paketleme:

- [[systemd-services]] — `victus-max-daemon.service` ve D-Bus etkinleştirilebilir servis tanımları.
- [[polkit-dbus-security]] — `org.hp.omen.conf` güvenlik politikası, `omen-hw` kullanıcı grubu ve root yetkilendirmesi.
- [[udev-device-rules]] — `99-victus-max.rules` ile `/dev/cpu/*/msr` ve evdev girdi aygıt izinleri.
- [[packaging-and-dkms]] — Cargo ikili optimizasyonları, Arch PKGBUILD, Nix Flakes ve DKMS otomatik derleme zinciri.
