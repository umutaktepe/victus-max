# Güç Otomasyonu ve AC/Pil Durum İzleyici

## Genel Bakış
`PowerAutomationService` (`src/victus-max-daemon/src/power_automation.rs`), cihazın güç kaynağı (AC Adaptör vs. Batarya) değişimlerini ve masaüstü ortamının (GNOME / KDE) güç tercihlerini izleyerek Victus Max donanım ayarlarını otomatik senkronize eden servistir.

## Entegrasyon Kanalları

### 1. PowerProfilesDaemon (PPD) D-Bus Senkronizasyonu
Modern Linux masaüstü ortamlarında (GNOME Hızlı Ayarlar paneli gibi) kullanıcı bir güç profili seçtiğinde, `net.hadess.PowerProfiles` servisi D-Bus üzerinde `PropertiesChanged` sinyali yayar.
`PowerAutomationService`, bu sinyali asenkron dinleyerek:
- `performance` seçildiğinde: HP WMI termal politikasını `Performance` moduna alır.
- `power-saver` seçildiğinde: HP WMI politikasını `Quiet` moduna geçirir.
- `balanced` seçildiğinde: HP WMI politikasını `Default` dengeli moduna ayarlar.

### 2. AC Adaptör Durum İzleme (AC Online Polling)
`/sys/class/power_supply/` altındaki güç kaynaklarını (örneğin `AC/online` veya `ADP1/online`) izler:
- **Fişten Çekildiğinde (Pilde):** Kullanıcıyı bilgilendirir, aşırı güç tüketimini ve fan gürültüsünü önlemek için otomatik tasarruf politikalarını önerir.
- **Fişe Takıldığında (AC Güçte):** Donanımın tam performans profillerine dönmesini sağlar.

## İlgili Bağlantılar
- Güç Servisi: [[power-service]]
- Platform Servisi: [[platform-service]]
- D-Bus Mimarisi: [[dbus-ipc-protocol]]
