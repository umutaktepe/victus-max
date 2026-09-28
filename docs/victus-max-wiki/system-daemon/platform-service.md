# Platform ve Genel Donanım Servisi (Platform Service)

## Genel Bakış
`PlatformService` (`src/victus-max-daemon/src/platform.rs`), HP Omen ve Victus cihazlarda genel donanım bakımını, pil sağlığı korumasını ve sistem tanı paketlerini yürüten servistir.

Servis, D-Bus üzerinde `org.hp.omen.Platform` arayüzünü sunar.

## Temel Fonksiyonlar

### 1. Pil Bakım Modu (Battery Care Limit)
Lityum bataryaların ömrünü uzatmak için şarj tavanını sınırlama (örneğin %80):
- HP WMI ACPI arayüzü veya `/sys/class/power_supply/BAT*/charge_control_limit_max` üzerinden şarj eşiğini ayarlar (`set_battery_care(limit)`).

### 2. Fan Temizleme Modu (Fan Cleaning)
Fanların ters yönde veya yüksek devirde aralıklı çalıştırılarak ızgaralarda biriken tozların tahliyesini sağlayan bakım rutini (`run_fan_cleaning()`).

### 3. Hızlı Teşhis Paketi (Triage Bundle)
Sistemde bir donanım veya sürücü sorunu yaşandığında; dmesg, journalctl, WMI çıktıları, EC dump ve donanım özelliklerini tek bir arşivde toplayarak geliştiriciye iletmeyi sağlayan hata ayıklama motoru (`generate_triage_bundle()`).

### 4. Overlay Tetikleme
[[hotkey-monitor]] tarafından algılanan Shift+F2 veya komut satırından gelen isteklerle oyun içi HUD penceresini ([[quick-hud-overlay]]) görünür kılan `toggle_overlay()` işlevi.

## İlgili Bağlantılar
- Kısayol İzleyici: [[hotkey-monitor]]
- Oyun İçi HUD: [[quick-hud-overlay]]
- D-Bus Arayüzü: [[dbus-ipc-protocol]]
