# SysMon Telemetri D-Bus Arayüzü (`org.hp.omen.SysMon`)

## Arayüz Tanımı
- **D-Bus Nesne Yolu:** `/org/hp/omen/SysMon`
- **Arayüz Adı:** `org.hp.omen.SysMon`
- **Uygulayıcı Servis:** [[sysmon-service]]

## Metotlar (Methods)

| Metot İmzası | Parametreler | Dönüş Tipi | Açıklama |
| :--- | :--- | :--- | :--- |
| `get_diagnostics()` | Yok | `String` (JSON) | Detaylı donanım sensör dökümünü anlık döner. |
| `get_hardware_specs()` | Yok | `String` (JSON) | Cihazın teknik özelliklerini (`HardwareSpecs`) döner. |
| `generate_diagnostic_report()` | Yok | `String` | Hata ayıklama için metin tabanlı sistem raporu üretir. |
| `generate_rgb_issue()` | Yok | `String` | RGB sürücü hataları için GitHub sorun taslağı oluşturur. |

## Sinyaller (Signals)

| Sinyal Adı | Argümanlar | Açıklama |
| :--- | :--- | :--- |
| `telemetry_updated` | `json_stats: &str` | Her 3 saniyede bir tetiklenir; tüm sistem sensör verilerini ([[system-telemetry-spec]]) tek bir JSON nesnesi halinde yayınlar. |

## İlgili Bağlantılar
- Servis Katmanı: [[sysmon-service]]
- Veri Şeması: [[system-telemetry-spec]]
- Tüketici İstemciler: [[quick-hud-overlay]], [[gui-application]]
