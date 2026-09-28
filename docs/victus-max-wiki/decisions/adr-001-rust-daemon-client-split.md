# ADR-001: Rust Tabanlı Ayrık Daemon-İstemci Mimarisi (Privilege Separation)

## Durum
Kabul Edildi

## Tarih
2024-05-15 (Revizyon: 2026-09-29)

## Bağlam
HP Omen, Victus ve Transcend serisi dizüstü bilgisayarlarda fan eğrilerinin güncellenmesi, MSR (Model-Specific Register) üzerinden işlemci voltajının ayarlanması, WMI/ACPI çağrıları yapılması ve gömülü denetleyiciye (EC) erişim doğrudan `root` yetkisi gerektirmektedir.

Projenin önceki nesil uygulaması olan Python tabanlı *OmenCtl*, tüm grafik arayüzü ve komut satırı mantığını doğrudan `root` veya `sudo` altında çalıştırmaktaydı. Bu durum aşağıdaki kritik sorunları doğurmaktaydı:
1. **Güvenlik Riski:** GTK gibi devasa grafik kütüphanelerinin ve X11/Wayland soketlerinin root olarak çalıştırılması sistem güvenliğini riske atmaktadır.
2. **Kaynak Tüketimi:** Python çalışma zamanı ve kütüphaneleri arka planda 50MB+ RAM tüketmekte ve sistem genelinde belirgin CPU gecikmelerine neden olmaktaydı.
3. **Süreç İzolasyonu Eksikliği:** Kullanıcı arayüzünde oluşan bir çökme veya donma, fan kontrol döngüsünü durdurmakta ve donanımın termal korumasız kalmasına yol açmaktaydı.

## Alternatifler
- **Alternatif A (Monolitik Sudo GUI):** Grafik uygulamasını `pkexec` veya `sudo` ile doğrudan root yetkisiyle başlatmak. (Güvenlik ve modern Wayland masaüstü entegrasyonu standartları gereğince reddedildi).
- **Alternatif B (Setuid C Yardımcı Binary):** Yalnızca donanım yazımı yapan küçük C programlarına setuid biti vermek. (Bakım zorluğu, sinyal ve IPC senkronizasyon yetersizliği nedeniyle reddedildi).
- **Alternatif C (Ayrık Rust Daemon + D-Bus IPC):** Donanım yetkisini tek bir arka plan servisinde (`omen-space-daemon`) toplamak ve kullanıcı arayüzlerini (`omen-gui`, `omen-cli`, `omen-overlay`, `omen-tray`) standart unprivileged kullanıcı oturumunda çalıştırmak.

## Karar
Sistem **İstemci-Sunucu (Client-Server) / Ayrık Ayrıcalık (Privilege Separation)** modeline geçirilmiştir:
1. **Çekirdek Servis:** Root yetkisiyle bir `systemd` servisi olarak çalışan [[daemon-overview]], Linux System Bus üzerinde `org.hp.omen` adıyla D-Bus nesnelerini yayınlar.
2. **Güvenli IPC:** Süreçler arası iletişimde sıfır maliyetli ve tip güvenli asenkron Rust kütüphanesi olan `zbus` kullanılmıştır. Ortak tipler ve D-Bus proxy arayüzleri [[dbus-ipc-protocol]] üzerinden `omen-types` kütüphanesinde paylaştırılmıştır.
3. **Erişim Kontrolü:** Yalnızca `omen-hw` veya `wheel` grubundaki yerel kullanıcılara izin veren Polkit ve D-Bus kural seti ([[polkit-dbus-security]]) yapılandırılmıştır.
4. **Hafif İstemciler:** [[gui-application]], [[quick-hud-overlay]], [[system-tray]] ve [[command-line-interface]] tamamen unprivileged kullanıcı yetkileriyle çalışır.

## Sonuçlar
### Olumlu
- **Düşük Kaynak Tüketimi:** Rust derlemesi sayesinde daemon bellek kullanımı <5MB RAM düzeyine inmiş, ikili dosya boyutu ~3MB seviyesine çekilmiştir.
- **Yüksek Güvenlik:** Arayüz katmanındaki hiçbir kod doğrudan donanım yazma hakkına sahip değildir. Tüm parametreler D-Bus uç noktasında katı doğrulamadan geçer.
- **Kararlılık:** Grafik arayüz kapansa veya çökse dahi donanım koruma döngüleri ([[fan-service]], [[power-service]]) arka planda kesintisiz çalışmaya devam eder.

### Olumsuz / Trade-off
- D-Bus serileştirme/ters-serileştirme (IPC) ek bir soyutlama katmanı getirmiştir.
- Yeni bir donanım yeteneği eklendiğinde hem daemon uç noktasının hem de `omen-types` D-Bus proxy sözleşmesinin güncellenmesi gerekmektedir.

## İlgili Bağlantılar
- Mimari Uygulayıcı: [[daemon-overview]]
- İletişim Protokolü: [[dbus-ipc-protocol]]
- Güvenlik Politikası: [[polkit-dbus-security]]
- Arayüz İstemcisi: [[gui-application]]
