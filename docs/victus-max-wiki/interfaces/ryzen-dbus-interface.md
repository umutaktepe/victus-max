# AMD Ryzen D-Bus Arayüzü (`org.hp.omen.Ryzen`)

## Arayüz Tanımı
- **D-Bus Nesne Yolu:** `/org/hp/omen/Ryzen`
- **Arayüz Adı:** `org.hp.omen.Ryzen`
- **Uygulayıcı Servis:** [[ryzen-service]]

## Metotlar ve Kontroller

| Metot İmzası | Parametreler | Dönüş Tipi | Açıklama |
| :--- | :--- | :--- | :--- |
| `set_limits(stapm: u32, fast: u32, slow: u32)` | Güç sınırları (Miliwatt cinsinden) | `String` | AMD SMU paket güç limitlerini ayarlar. |
| `set_tctl_temp(temp: u32)` | Sıcaklık eşiği (°C) | `String` | Tepe sıcaklık sınırını belirler. |
| `set_curve_optimizer(co: i32)` | CO ofseti (örn. `-20`) | `String` | Tüm çekirdeklere Curve Optimizer uygular. |
| `get_ryzen_state()` | Yok | `String` (JSON) | Mevcut SMU parametrelerini ve aile bilgisini döner. |

## Yapılandırma Dosyası (`/etc/omen-space/ryzen.json`)

```json
{
  "stapm_limit": 45000,
  "fast_limit": 65000,
  "slow_limit": 54000,
  "tctl_temp": 90,
  "all_core_co": -15
}
```

## İlgili Bağlantılar
- Servis Katmanı: [[ryzen-service]]
- Donanım Katmanı: [[amd-ryzen-smu]]
- Çakışma Denetimi: [[conflict-detector]]
