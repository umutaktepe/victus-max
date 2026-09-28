# Undervolt D-Bus Arayüzü (`org.hp.omen.Undervolt`)

## Arayüz Tanımı
- **D-Bus Nesne Yolu:** `/org/hp/omen/Undervolt`
- **Arayüz Adı:** `org.hp.omen.Undervolt`
- **Uygulayıcı Servis:** [[undervolt-service]]

## Metotlar (Methods)

| Metot İmzası | Parametreler | Dönüş Tipi | Açıklama |
| :--- | :--- | :--- | :--- |
| `set_offset(plane: &str, offset_mv: i32)` | Düzlem adı (`"core"`, `"cache"`, `"gpu"` vb.), Negatif mV (örn. `-50`) | `String` | İlgili Intel voltaj düzlemine ofset yazar. |
| `set_tcc_offset(val: i32)` | Sıcaklık ofset derecesi (`0..15`) | `String` | MSR 0x1A2 TCC sıcaklık tavanını düşürür. |
| `get_state()` | Yok | `String` (JSON) | Kayıtlı ve canlı MSR ofset durumunu döner. |

## JSON Yanıt Formatı (`get_state`)

```json
{
  "core": -60,
  "cache": -60,
  "gpu": 0,
  "uncore": 0,
  "analogio": 0,
  "tcc_offset": 5,
  "undervolt_supported": true
}
```

## İlgili Bağlantılar
- Servis Katmanı: [[undervolt-service]]
- Donanım Arabirimi: [[cpu-msr-undervolt]]
- Mimari Karar: [[adr-004-native-msr-and-smu-mailbox-tuning]]
