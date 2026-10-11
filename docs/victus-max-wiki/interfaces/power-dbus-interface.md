# Güç D-Bus Arayüzü ve Konfigürasyon Şeması (`org.hp.omen.Power`)

## Arayüz Tanımı
- **D-Bus Nesne Yolu:** `/org/hp/omen/Power`
- **Arayüz Adı:** `org.hp.omen.Power`
- **Uygulayıcı Servis:** [[power-service]]

## Metotlar (Methods)

| Metot İmzası | Parametreler | Dönüş Tipi | Açıklama |
| :--- | :--- | :--- | :--- |
| `set_power_profile(profile: &str)` | `"power-saver"`, `"balanced"`, `"performance"` | `String` | ACPI termal güç profilini ayarlar. |
| `get_power_profile()` | Yok | `String` (JSON) | Yüksek frekanslı (1.5s/3s) yoklama için hafif metot; aktif güç profilini ve yapılandırma özetini SMU posta kutusuna dokunmadan döner. |
| `get_power_limits_info()` | Yok | `String` (JSON) | İşlemci mimarisi (`is_amd`, `cpu_tier`, `stapm_fast_linked`), dinamik aralıklar (`ranges`) ve `RYZENADJ_MUTEX` ile seri hale getirilmiş canlı RAPL/RyzenAdj sınırlarını (`pl1_w`, `pl2_w`, `stapm_w`, `fast_ppt_w`, `slow_ppt_w`, `tctl_temp`) döner; Zen 4/5 sistemlerde `stapm_fast_linked: true` ile STAPM ve PPT Fast donanım bağını bildirir. |
| `set_power_limits(enabled: bool, pl1: i32, pl2: i32)` | `enabled`, `pl1` (Watt), `pl2` (Watt) | `String` | Intel'de RAPL PL1/PL2 sınırlarını ayarlar (geriye dönük uyumlu 2 parametreli metot). |
| `set_ryzen_power_limits(enabled: bool, stapm: i32, slow: i32, fast: i32)` | `enabled`, `stapm` (W), `slow` (W), `fast` (W) | `String` | AMD Ryzen işlemcilerde `STAPM`, `PPT Slow` ve `PPT Fast` limitlerini mimari nesline göre normalize ederek (`normalize_ryzen_limits`) güvenli biçimde ayarlar. |
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
  "pl1_w": 48,
  "slow_ppt_w": 43,
  "pl2_w": 48,
  "pl_enabled": true,
  "gpu_w": 115
}
```

## İlgili Bağlantılar
- Mimari Karar: [[adr-007-adaptive-ryzen-power-limits]]
- Servis Katmanı: [[power-service]]
- Güç Otomasyonu: [[power-automation-service]]
- D-Bus Protokolü: [[dbus-ipc-protocol]]
