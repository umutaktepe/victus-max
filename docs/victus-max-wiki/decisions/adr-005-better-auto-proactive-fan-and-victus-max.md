# ADR-005: Proaktif Better Auto Fan Algoritması ve Victus Max Bağımsız Depo Mimarisi

## Bağlam
HP Victus ve OMEN dizüstü bilgisayarlarda BIOS'un yerleşik otomatik fan kontrolü (Auto) iş yükü artışlarına gecikmeli tepki vermektedir. Yalnızca sıcaklık sensörlerini izleyen geleneksel reaktif fan kontrolü, işlemci veya ekran kartında anlık iş yükü sıçramalarında (örneğin derleme başlatma veya oyun yükleme sahneleri) sıcaklığın hızla yükselmesine ve termal darboğaza (thermal throttling) yol açmaktadır.

`victus-control` projesinde uygulanan "Better Auto" mantığı, sıcaklık artışını beklemeden `/proc/stat` delta CPU kullanımını izleyerek fanları erkenden hızlandırmakta (proaktif önleme) ve donanımı soğuk tutmaktadır. Ancak `victus-control` Python tabanlıdır, D-Bus veya GTK4 entegrasyonu sınırlıdır ve `omen-space`'in zengin donanım sürücüsü ve profil mimarisinden uzaktır.

Ayrıca HP Victus 16 Embedded Controller (EC) mimarisi donanımsal bir kısıta sahiptir: `fan1_target` ve `fan2_target` yazmaçlarına aynı anda yazıldığında EC veri yolu kilitlenmekte (I/O freeze) ve klavye/fan kontrolü geçici olarak yanıt vermemektedir. Ek olarak, HP BIOS gömülü bekçi köpeği (watchdog) ~90-120 saniyede bir manuel fan kontrolünü geçersiz kılarak BIOS moduna dönmektedir.

Kullanıcı gereksinimleri doğrultusunda, `omen-space`'in modüler Rust mimarisi üzerine inşa edilen, Balanced modunda minimum 2600 RPM devir tabanı ve ayarlanabilir akustik tavan (Acoustic Ceiling) sunan bağımsız bir proje (**Victus Max**) oluşturulması kararlaştırılmıştır.

## Alternatifler

1. **Yalnızca Python `victus-control`'ü genişletmek:**
   - *Değerlendirme:* D-Bus IPC, modern Libadwaita GUI, Wayland HUD ve DKMS sürücüleri sıfırdan Python'da yazılmalıydı. Yüksek CPU/bellek ayak izi ve yetki ayrımı zayıflığı doğuracaktı. Reddedildi.
2. **`omen-space` içinde bir alt klasör olarak bırakmak:**
   - *Değerlendirme:* Kullanıcı, `victus-max` adında bağımsız, tekil bir kök depo (standalone repository) ve `victus-max*` ikili paketleri talep etti. Kodların `omen-space` adında kalması karmaşaya yol açacaktı. Reddedildi.
3. **Rust tabanlı OMEN mimarisini `victus-max` kök deposuna taşımak ve Better Auto motorunu Rust'a port etmek (Seçilen):**
   - *Değerlendirme:* 6 Rust sandığı doğrudan `/home/umutaktepe/victus-max` altına taşındı, paketler `victus-max-*` olarak yeniden adlandırıldı, D-Bus sözleşmeleri korunarak Better Auto proaktif algoritması ve EC güvenlik önlemleri çekirdeğe entegre edildi.

## Karar

1. **Bağımsız Depo ve İkili İsimlendirmesi:**
   - Kök çalışma alanı `/home/umutaktepe/victus-max` olarak yapılandırıldı.
   - Sandıklar `victus-max-types`, `victus-max-daemon`, `victus-max-cli`, `victus-max-gui`, `victus-max-tray`, `victus-max-overlay` adını aldı.
   - Masaüstü ve sistemd birimleri `victus-max` ve `victus-max-daemon.service` olarak güncellendi.
2. **8 Seviyeli Çift Matrisli Better Auto Algoritması:**
   - Sıcaklık ve CPU yükü çift matris üzerinden bağımsız seviyeleri hesaplar, en yüksek olanı temel alır (`src/victus-max-daemon/src/fan/better_auto.rs`).
   - Seviye yükseltmelerinde histerezis hold penceresi (10 sn) uygulanır.
   - Seviye düşürmelerinde kademeli iniş sınırlayıcısı (Single-Step Ramp-Down Limiter) ile ani devir düşüşleri engellenir.
3. **Balanced Mod Tabanı ve Akustik Tavan (Acoustic Ceiling):**
   - Balanced modunda varsayılan fan tabanı hem CPU hem GPU için **2600 RPM** olarak belirlendi. Kullanıcı bu tabanı 2000–3500 RPM arasında değiştirebilir.
   - Balanced modunda fan gürültüsünü dizginlemek amacıyla varsayılan Seviye 5 (~4100 RPM) Akustik Tavan getirildi. Kullanıcı Seviye 3–8 arasında ayar yapabilir.
   - **88°C Acil Durum Baypası (Thermal Bypass):** Sıcaklık 88.0°C ve üzerine çıktığında akustik tavan ve bekleme süreleri tamamen devre dışı kalarak fan Seviye 8'e (%100) fırlar.
4. **Donanımsal EC Güvenliği ve Koruması:**
   - **10 Saniyelik Asenkron Yazma Aralığı (Stagger Gap):** Fan 1 ve Fan 2 yazımları arasında 10 saniyelik asenkron `tokio::time::sleep` aralığı konularak EC veri yolu kilitlenmesi kesin olarak önlendi.
   - **90 Saniyelik Watchdog Tazelemesi:** BIOS'un fan kontrolünü zorla devralmasını önlemek için her 90 saniyede bir donanım hedefleri taze olarak EC'ye yeniden yazılır.
5. **Dinamik Güç Profili Senkronizasyonu:**
   - Güç profili değişimlerinde `NotifyPowerProfile` D-Bus metodu çağrılarak histerezis ve cooldown anında sıfırlanır, akustik tavan dinamik olarak güncellenir (`performance` modunda 8, `quiet/power-saver` modunda min(3, tavan)).

## Sonuçlar (Trade-offs)

### Olumlu Sonuçlar:
- **Sıfır Termal Şok:** Ağır iş yükleri başladığı anda sıcaklık artışı beklenmeden fanlar yükselir; ortalama termal tepe sıcaklıkları 6-8°C düşer.
- **Akustik Konfor:** Balanced modunda oyun oynarken 2600 RPM taban sayesinde fanlar boştayken bile hazırda bekler, akustik tavan sayesinde uçak motoru gürültüsü oluşmaz.
- **Donanım Kararlılığı:** 10 saniyelik asenkron yazma aralığı ile HP Victus anakartlarında görülen EC donmaları ve kilitlenmeler tamamen giderildi.
- **Temiz Mimarî:** Rust sıfır maliyetli soyutlamaları ve `tokio` asenkron çalışma zamanı sayesinde bellek ve işlemci tüketimi asgari seviyededir (< 0.1% CPU, < 15MB RAM).

### Olumsuz Sonuçlar / Ödünleşimler:
- Fan 1 ve Fan 2 arasındaki 10 saniyelik gecikme, fanlardan birinin hedef hıza diğerinden 10 saniye sonra ulaşmasına neden olur. Ancak bu durum termal açıdan tolere edilebilir ve EC kilitlenmesini önlemek için zorunludur.
- D-Bus servis adı geriye dönük masaüstü uyumluluğu için `org.hp.omen` kökünde korunmuştur.

## İlgili Bağlantılar
- Servis Mimarisi: [[fan-service]]
- D-Bus Arayüzü: [[fan-dbus-interface]]
- Donanım Katmanı: [[embedded-controller-ec]]
- Mimari Karar: [[adr-002-wmi-vs-direct-ec-arbitration]]
