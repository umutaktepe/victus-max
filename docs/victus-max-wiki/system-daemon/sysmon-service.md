# Sistem Telemetrisi ve Donanım İzleme Servisi (SysMon)

## Genel Bakış
`SysMonInterface` ve telemetri altyapısı (`src/victus-max-daemon/src/sysmon/`), sistem donanım sensörlerini düşük ek yük ile sürekli izleyen ve istemcilere canlı telemetri akışı sağlayan bileşendir.

Servis, D-Bus üzerinde `org.hp.omen.SysMon` arayüzünü ([[sysmon-dbus-interface]]) sunar.

## Modüler Yapı (`src/sysmon/`)

Telemetri motoru bağımsız sorumluluklara ayrılmış 5 alt modülden oluşur:
- **`sensors.rs`:** Linux `hwmon` alt sistemini tarar; CPU çekirdek sıcaklıkları, fan devirleri (RPM) ve kasa sıcaklıklarını okur.
- **`gpu.rs`:** NVIDIA NVML / `nvidia-smi` veya açık kaynak sürücüler (Nouveau/AMDGPU) üzerinden GPU sıcaklığı, kullanım yüzdesi ve güç çekimini (W) çeker.
- **`stats.rs`:** `/proc/stat`, `/proc/meminfo` ve disk kullanım verilerini derleyerek genel sistem yükünü hesaplar.
- **`utils.rs`:** DMI bilgileri, BIOS sürümleri ve kernel detaylarını toplayan yardımcı fonksiyonlar.
- **`interface.rs`:** Toplanan verileri D-Bus üzerinden istemcilere sunan ve teşhis raporları üreten arayüz katmanı.

## Canlı Telemetri Yayını (`telemetry_updated`)
`victus-max-daemon` ana döngüsü, her 3 saniyede bir `fetch_system_stats` fonksiyonunu asenkron olarak çağırır:
- Toplanan veriler [[system-telemetry-spec]] (`SystemStats`) formatında JSON'a serileştirilir.
- D-Bus `telemetry_updated(json_stats)` sinyali ile tüm abonelere yayılır.
- Bu sinyal sayesinde [[quick-hud-overlay]] ve [[gui-application]] sıfır yoklama (zero-polling) ile anında güncellenir.

## Tanı ve Raporlama (Diagnostics Engine)
Servis aşağıdaki gelişmiş teşhis araçlarını içerir:
- **`generate_diagnostic_report()`:** BIOS, kernel, fan ve ACPI durumunu içeren kapsamlı sistem raporu.
- **`generate_rgb_issue()`:** RGB sürücüsü veya sysfs düğümlerinde hata olduğunda GitHub sorun formatında çıktı üreten teşhis aracı.

## İlgili Bağlantılar
- D-Bus Sözleşmesi: [[sysmon-dbus-interface]]
- Veri Şeması: [[system-telemetry-spec]]
- Arayüz Tüketicisi: [[quick-hud-overlay]], [[gui-application]]
