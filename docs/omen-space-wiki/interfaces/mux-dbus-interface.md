# MUX Switch D-Bus Arayüzü (`org.hp.omen.Mux`)

## Arayüz Tanımı
- **D-Bus Nesne Yolu:** `/org/hp/omen/Mux`
- **Arayüz Adı:** `org.hp.omen.Mux`
- **Uygulayıcı Servis:** [[mux-service]]

## Metotlar (Methods)

| Metot İmzası | Parametreler | Dönüş Tipi | Açıklama |
| :--- | :--- | :--- | :--- |
| `set_gpu_mode(mode: &str)` | `"discrete"`, `"hybrid"` | `String` | GPU ekran yönlendirme modunu değiştirir. Başarılı ise `"OK_REBOOT_REQUIRED"` döner. |
| `get_gpu_info()` | Yok | `String` (JSON) | Aktif mod, ekran çıkışları ve donanım MUX durumunu döner. |

## JSON Çıktı Şeması (`get_gpu_info`)

```json
{
  "active_mode": "discrete",
  "backend": "hp-wmi",
  "has_mux_hardware": true,
  "displays": [
    {
      "connector": "eDP-1",
      "status": "connected",
      "driven_by": "nvidia"
    }
  ]
}
```

## İlgili Bağlantılar
- Servis Katmanı: [[mux-service]]
- Çekirdek Arabirimi: [[hp-wmi-driver]]
- D-Bus Protokolü: [[dbus-ipc-protocol]]
