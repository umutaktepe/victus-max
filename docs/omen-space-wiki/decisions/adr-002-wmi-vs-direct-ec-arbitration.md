# ADR-002: WMI ve Doğrudan EC (Embedded Controller) Tahkim Stratejisi

## Durum
Kabul Edildi

## Tarih
2024-07-20 (Revizyon: 2026-09-29)

## Bağlam
HP dizüstü bilgisayarlarda fan hızlarını yönetmek ve sensör verilerini okumak için iki temel donanım kanalı mevcuttur:
1. **WMI ACPI Arayüzü (`hp-wmi`):** Standart BIOS çağrıları (`hp_wmi_perform_query`). Güvenlidir ancak bazı Victus ve Transcend modellerinde özel fan eğrilerini veya ince devir ayarlarını desteklemez.
2. **Gömülü Denetleyici IO (`/sys/kernel/debug/ec/ec0/io`):** EC bellek adreslerine (`0x2E`, `0x2F`, `0x34`, `0x35`) doğrudan bayt düzeyinde erişim. Çok hızlı ve hassastır.

Ancak saha testlerinde belirli anakartlarda (`8c58`, `8d24`) ve belirli kasa modellerinde (`16t-ah0`, `16-ah0`, `16-ap0`, `17t-ah0`, `17-ah0`, `transcend 14`) doğrudan EC yazımlarının gömülü denetleyici firmware'inde kilitlenmeye, klavye Caps Lock ışığının panik modunda yanıp sönmesine ve sistemin aniden donmasına neden olduğu saptanmıştır.

Tersine, `8A42`, `8A43` ve `8E35` anakart kodlu modellerde ise standart Linux `hwmon` alt sisteminde hiçbir `fan[1-2]_input` sensörü bulunmamakta; fan hızları yalnızca EC üzerindeki `0x2E` ve `0x2F` görev döngüsü (duty-cycle) kayıtlarından okunabilmektedir.

## Alternatifler
- **Alternatif A (Salt WMI Kullanımı):** Yalnızca WMI ACPI çağrılarını desteklemek. (hwmon desteği olmayan modellerde fan hızlarının okunamaması ve gelişmiş fan eğrilerinin uygulanamaması sebebiyle reddedildi).
- **Alternatif B (Herkese Açık Doğrudan EC Yazımı):** Tüm modellerde EC portuna doğrudan yazmak. (Belirtilen güvensiz modellerde sistemin çökmesi ve donanım kararsızlığı nedeniyle kesinlikle reddedildi).
- **Alternatif C (Model Tabanlı Hibrit Tahkim & Güvenlik Duvarı):** DMI bilgisine göre çalışma anında dinamik bir donanım matrisi denetimi yaparak doğrudan EC erişimini şartlı yetkilendirmek.

## Karar
Sistem [[embedded-controller-ec]] modülü üzerinden katı bir **Donanım Tahkim Katmanı (Hardware Arbitration Layer)** kurmuştur:
1. **Güvensiz Donanım Kara Listesi:** DMI `product_name` ve `board_name` taranır. Tanımlı `UNSAFE_MODELS` ve `UNSAFE_BOARDS` listesinde yer alan sistemlerde doğrudan EC yazımları tamamen engellenir ve fan kontrolü standart [[hp-wmi-driver]] katmanına yönlendirilir.
2. **hwmon Fallback Beyaz Listesi:** Anakart kimliği `8A42`, `8A43` veya `8E35` olan sistemlerde, hwmon fan sensörü bulunamadığında EC üzerinden devir okuma (`needs_ec_fallback`) otomatik olarak devreye sokulur.
3. **Kademeli Erişim Denetimi:** Daemon ilk başladığında `/sys/kernel/debug` dizininin mount durumunu kontrol eder, gerekiyorsa debugfs'i güvenli bağlar ancak donanım onayından geçmeyen hiçbir modelde EC yazma kanalı açılmaz.

## Sonuçlar
### Olumlu
- **Kritik Çökme Engellemesi:** Güvensiz modellerde Caps Lock kilitlenmeleri ve firmware panikleri %100 oranında önlenmiştir.
- **Geniş Donanım Uyumluluğu:** Standart hwmon sensörü eksik olan modern Victus modellerinde fan telemetrisi kesintisiz sağlanmıştır.
- **Şeffaf Yapılandırma:** Hangi anakartın hangi modu kullandığı [[board-capabilities-matrix]] (`boards.json`) üzerinde açıkça izlenebilir hale getirilmiştir.

### Olumsuz / Trade-off
- Yeni HP laptop modelleri piyasaya çıktıkça kara liste ve beyaz listenin DMI eşleşmeleri güncel tutulmalıdır.
- Kullanıcıların debugfs erişimi gerektiren durumlar için çekirdek konfigürasyonlarında `CONFIG_EC_SYS=m` seçeneğinin bulunması gereklidir.

## İlgili Bağlantılar
- Donanım Katmanı: [[embedded-controller-ec]]
- Sürücü Entegrasyonu: [[hp-wmi-driver]]
- Servis Tüketicisi: [[fan-service]]
- Donanım Matrisi: [[board-capabilities-matrix]]
