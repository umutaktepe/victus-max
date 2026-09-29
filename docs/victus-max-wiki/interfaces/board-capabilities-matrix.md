# Donanım Kabiliyet Matrisi Şartnamesi (`boards.json`)

## Genel Bakış
HP Omen, Victus ve Transcend serisi onlarca farklı anakart mimarisine sahiptir. Her anakartın fan denetleyicisi, RGB bölgeleri, MUX switch donanımı ve undervolt kilitleri farklılık gösterir.

Victus Max, bu çeşitliliği yönetmek için `boards.json` (`src/victus-max-daemon/src/boards.json`) veri tabanını ve `capabilities.rs` (`src/victus-max-daemon/src/capabilities.rs`) motorunu kullanır.

## Kabiliyet Sınıfları (`LinuxCapabilityClass`)

Sistem, DMI anakart kimliğine göre cihazı 4 kabiliyet seviyesinden birine sınıflandırır:
1. **`FullControl`:** Hem WMI hem de EC doğrudan fan kontrolü, özel fan eğrisi, voltaj düşürme ve güç limitleri tam desteklenir.
2. **`ProfileOnly`:** Doğrudan EC erişimi riskli veya kısıtlıdır; fanlar yalnızca BIOS ACPI profilleri (Quiet, Balanced, Performance) üzerinden yönetilebilir.
3. **`TelemetryOnly`:** Donanım yazmaçları kilitlidir; yalnızca sensör okuma ve izleme yapılabilir.
4. **`UnsupportedControl`:** Desteklenmeyen veya tanımlanmamış anakart mimarisi.

## `ModelCapabilities` Veri Şeması

`boards.json` içerisindeki her bir cihaz kaydı şu yapıyı takip eder:

```json
{
  "product_id": "8A14",
  "model_name": "OMEN 15 (2020) Intel",
  "model_year": 2020,
  "family": "Legacy",
  "supports_fan_control_wmi": true,
  "supports_fan_control_ec": true,
  "supports_fan_curves": true,
  "supports_independent_fan_curves": true,
  "supports_rpm_readback": true,
  "fan_zone_count": 2,
  "max_fan_speed_percent": 100,
  "min_fan_speed_percent": 0,
  "supports_performance_modes": true,
  "performance_modes": ["Default", "Performance", "Cool"],
  "has_mux_switch": false,
  "supports_gpu_power_boost": true,
  "has_keyboard_backlight": true,
  "has_four_zone_rgb": true,
  "has_per_key_rgb": false,
  "supports_undervolt": true,
  "supports_tcc_offset": true,
  "supports_power_limits": true,
  "notes": "Well-tested model with full WMI BIOS support"
}
```

## Bilinmeyen Modellerde Fallback Davranışı
Eğer cihazın DMI anakart kodu `boards.json` içinde bulunamazsa, sistem varsayılan güvenli parametrelerle (`ModelCapabilities::default()`) başlatılır:
- Doğrudan EC yazımları güvenlik gerekçesiyle kapatılır (`supports_fan_control_ec = false`).
- Standart WMI termal profilleri etkinleştirilir.
- Kullanıcıya sistemin test edilmemiş bir anakartta çalıştığı bilgisi verilir.

## İlgili Bağlantılar
- Donanım Katmanı: [[embedded-controller-ec]]
- Fan Servisi: [[fan-service]]
- Mimari Karar: [[adr-002-wmi-vs-direct-ec-arbitration]]
