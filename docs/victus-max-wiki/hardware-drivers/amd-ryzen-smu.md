# AMD Ryzen SMU (System Management Unit) Güç ve Voltaj Ayarı

## Genel Bakış
AMD Ryzen mobil ve masaüstü işlemcilerde güç limitleri, sıcaklık hedefleri ve çekirdek voltaj eğrileri (Curve Optimizer); yonga üzerindeki bağımsız bir yardımcı işlemci olan **Sistem Yönetim Birimi (System Management Unit - SMU)** tarafından kontrol edilir.

OMEN Space (`src/victus-max-daemon/src/ryzen.rs`), harici bir C ikili dosyası olan `ryzenadj` aracına bağımlı kalmadan, SMU posta kutusu (Mailbox) protokolünü doğrudan Rust içinde yerel olarak uygular.

## Desteklenen AMD Aileleri ve Otomatik Algılama

`ryzen.rs` CPUID ve DMI parametreleri üzerinden işlemci ailesini dinamik olarak tespit eder:
- **Zen 1 / Zen+:** Raven Ridge, Picasso, Dali
- **Zen 2:** Renoir, Lucienne, Van Gogh, Mendocino, Matisse
- **Zen 3 / Zen 3+:** Cezanne, Barcelo, Rembrandt, Vermeer
- **Zen 4:** Phoenix, Hawk Point, Raphael / Dragon Range
- **Zen 5:** Strix Point, Strix Halo, Fire Range

Her işlemci ailesinin SMU mesaj ID'leri (Message IDs) ve argüman formatları farklılık gösterir. Modül bu aileye göre doğru SMU register adreslerini eşler.

## Kontrol Edilen SMU Parametreleri

| Parametre | Değişken Adı | Açıklama |
| :--- | :--- | :--- |
| **STAPM Limit** | `stapm_limit` | Kasa yüzey sıcaklığını korumak için tasarlanmış sürdürülebilir sürekli güç limiti (Miliwatt veya Watt cinsinden). |
| **Fast PPT** | `fast_limit` | Anlık yük patlamalarında CPU'nun izin verilen tepe paket güç tavanı (Package Power Tracking). |
| **Slow PPT** | `slow_limit` | Termal doyum sonrasında CPU'nun stabilize olduğu uzun vadeli güç tavanı. |
| **Tctl Sıcaklık Limiti** | `tctl_temp` | İşlemcinin termal kısmaya girmeden önce ulaşabileceği maksimum hedef sıcaklık derecesi. |
| **Curve Optimizer** | `all_core_co` | Tüm çekirdekler için negatif voltaj eğrisi ofseti (örneğin -10 ile -30 arası güvenli adım). |

## Posta Kutusu İletişim Protokolü (Mailbox Protocol)
1. **İstek Yazımı:** Mesaj ID'si ve argümanları işlemcinin MSR veya SMU yazmaç adreslerine yerleştirilir.
2. **Yürütme Tetiklemesi:** SMU komut register'ına bayt yazılarak istek aktif edilir.
3. **Cevap Bekleme (Polling):** SMU'nun isteği tamamladığını belirten `SMU_STATUS` yanıtı gelene kadar mikro-gecikmeli döngü çalıştırılır.
4. **Hata Yakalama:** Desteklenmeyen veya reddedilen SMU komutlarında donanım güvenliği için işlem iptal edilir.

## Süreç Çakışmalarının Önlenmesi
Arka planda harici bir `ryzenadj` döngüsü veya benzeri güç aracı çalıştığında SMU posta kutusunda yarış durumları oluşabilir. [[conflict-detector]] başlangıçta bu servisleri tarayarak sistemi uyarır.

## İlgili Bağlantılar
- Mimari Karar: [[adr-004-native-msr-and-smu-mailbox-tuning]]
- Servis Katmanı: [[ryzen-service]]
- D-Bus Sözleşmesi: [[ryzen-dbus-interface]]
- Çakışma Denetimi: [[conflict-detector]]
