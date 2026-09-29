# Güç D-Bus Arayüzü ve Konfigürasyon Şeması (`org.hp.omen.Power`)

## Arayüz Tanımı
- **D-Bus Nesne Yolu:** `/org/hp/omen/Power`
- **Arayüz Adı:** `org.hp.omen.Power`
- **Uygulayıcı Servis:** [[power-service]]

## Metotlar (Methods)

| Metot İmzası | Parametreler | Dönüş Tipi | Açıklama |
| :--- | :--- | :--- | :--- |
| `set_power_profile(profile: &str)` | `"power-saver"`, `"balanced"`, `"performance"` | `String` | ACPI termal güç profilini ayarlar. |
| `get_power_profile()` | Yok | `String` (JSON) | Aktif güç profili ve sınırlarını döner. |
| `set_power_limits(enabled: bool, pl1: i32, pl2: i32)` | `enabled`, `pl1` (Watt), `pl2` (Watt) | `String` | Intel RAPL PL1 ve PL2 sınırlarını ayarlar. |
| `set_app_profiles_enabled(enabled: bool)` | `true` / `false` | `String` | Süreç bazlı otomatik profil tetiklemeyi açar/kapatır. |

## Konfigürasyon Dosya Şeması (`/etc/victus-max/power.json`)
*(Not: Eski `/etc/omen-space/power.json` yoluna geriye dönük tam uyumluluk desteklenir)*

```json
{
  "power_profile": "performance",
  "app_profiles_enabled": true,
  "app_profiles": {},
  "undervolt_mv": -75,
  "tcc_offset": 5,
  "pl1_w": 55,
  "pl2_w": 95,
  "pl_enabled": true,
  "gpu_w": 115
}
```

## İlgili Bağlantılar
- Servis Katmanı: [[power-service]]
- Güç Otomasyonu: [[power-automation-service]]
- D-Bus Protokolü: [[dbus-ipc-protocol]]
