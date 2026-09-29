# Uygulama ve Oyun Profilleri Şartnamesi (`app_profiles.json`)

## Genel Bakış
Victus Max, kullanıcının belirlediği süreçler (oyunlar, simülatörler, 3D modelleme yazılımları) başladığında otomatik donanım profili tetiklemek için JSON tabanlı bir kural motoru kullanır.

Bu kurallar öncelikli olarak `/etc/victus-max/app_profiles.json` dosyasında saklanır (eski `/etc/omenspace/app_profiles.json` yoluna geriye dönük tam uyumluluk korunur) ve [[game-automation-service]] tarafından yürütülür.

## Şema ve JSON Yapısı

Dosya, `AppProfile` nesnelerinden oluşan bir dizidir:

```json
[
  {
    "process_name": "cs2",
    "power_profile": "performance",
    "fan_mode": "max"
  },
  {
    "process_name": "cyberpunk2077.exe",
    "power_profile": "performance",
    "fan_mode": "custom"
  },
  {
    "process_name": "code",
    "power_profile": "power-saver",
    "fan_mode": "auto"
  }
]
```

## Alan Açıklamaları

- **`process_name` (`String`):** Linux süreç tablosunda (`/proc/*/comm` veya `cmdline`) aranan ikili dosya adı. Hem yerel Linux binary isimleri (`cs2`) hem de Proton/Wine altındaki Windows yürütülebilir dosyaları (`*.exe`) desteklenir.
- **`power_profile` (`String`):** [[power-service]] tarafından kabul edilen geçerli güç profili (`"performance"`, `"balanced"`, `"power-saver"`).
- **`fan_mode` (`String`):** [[fan-service]] tarafından kabul edilen geçerli fan modu (`"auto"`, `"max"`, `"custom"` veya yüzde).

## D-Bus Yönetim Metotları (`org.hp.omen.AppProfiles`)
İstemciler bu dosyayı doğrudan düzenlemek yerine D-Bus arayüzünü kullanır:
- `add_profile(process_name, power_profile, fan_mode)`
- `remove_profile(process_name)`
- `get_profiles()`

## İlgili Bağlantılar
- Uygulayıcı Servis: [[game-automation-service]]
- Güç Servisi: [[power-service]]
- Fan Servisi: [[fan-service]]
- GUI Yönetimi: [[gui-application]]
