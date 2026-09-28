# Fan D-Bus Arayüzü ve Eğri Şeması (`org.hp.omen.Fan`)

## Arayüz Tanımı
- **D-Bus Nesne Yolu:** `/org/hp/omen/Fan`
- **Arayüz Adı:** `org.hp.omen.Fan`
- **Uygulayıcı Servis:** [[fan-service]]

## Metotlar (Methods)

| Metot İmzası | Parametreler | Dönüş Tipi | Açıklama |
| :--- | :--- | :--- | :--- |
| `set_fan_mode(mode: &str)` | `"auto"`, `"better_auto"`, `"max"`, `"custom"`, veya `"50"` (yüzde) | `String` (`"OK"`, `"FAIL"`) | Aktif fan çalışma modunu ayarlar. |
| `get_fan_mode()` | Yok | `String` | Aktif modu döner. |
| `get_fan_info()` | Yok | `String` (JSON) | Fan hızları, RPM değerleri, `min_fan_rpm` ve `acoustic_ceiling` sınırlarını döner. |
| `save_custom_curve(curve_json: &str)` | Özel eğri JSON metni | `String` (`"OK"`) | Özel fan eğrisini sisteme kaydeder. |
| `set_thermal_protection(enabled: bool)` | `true` / `false` | `String` (`"OK"`) | 95°C termal koruma devresini açar/kapatır. |
| `get_min_fan_rpm()` | Yok | `u32` | Better Auto için asgari devir tabanını döner (varsayılan 2600 RPM). |
| `set_min_fan_rpm(rpm: u32)` | `2000..=3500` | `bool` | Asgari fan devir tabanını ayarlar ve yapılandırmaya kaydeder. |
| `get_acoustic_ceiling()` | Yok | `u32` | Balanced modundaki akustik tavan seviyesini döner (3–8, varsayılan 5). |
| `set_acoustic_ceiling(level: u32)` | `3..=8` | `bool` | Akustik tavan seviyesini ayarlar ve yapılandırmaya kaydeder. |
| `notify_power_profile(profile: &str)` | `"balanced"`, `"performance"`, `"quiet"` vb. | `String` | Güç profili geçişini bildirir, histerezis/cooldown sıfırlar ve tavanı uyarlar. |

## Sinyaller (Signals)

| Sinyal Adı | Argümanlar | Açıklama |
| :--- | :--- | :--- |
| `thermal_protection_alert` | `active: bool` | Sıcaklık 95°C'yi aştığında (`true`) veya 82°C altına indiğinde (`false`) tetiklenir. |

## Özel Fan Eğrisi Veri Formatı (`curve_json`)

Özel fan eğrisi, sıcaklık (°C) ve PWM devir yüzdesi (%) ikililerinden oluşan bir JSON dizisidir:

```json
[
  [30.0, 0.0],
  [50.0, 25.0],
  [65.0, 45.0],
  [75.0, 70.0],
  [85.0, 100.0]
]
```

## İlgili Bağlantılar
- Servis Katmanı: [[fan-service]]
- Grafik Düzenleyici: [[fan-curve-editor-ui]]
- D-Bus Protokolü: [[dbus-ipc-protocol]]
- Mimari Kararlar: [[adr-002-wmi-vs-direct-ec-arbitration]], [[adr-005-better-auto-proactive-fan-and-victus-max]]
