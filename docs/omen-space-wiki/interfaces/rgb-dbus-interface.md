# RGB D-Bus Arayüzü ve Renk Şeması (`org.hp.omen.Rgb`)

## Arayüz Tanımı
- **D-Bus Nesne Yolu:** `/org/hp/omen/Rgb`
- **Arayüz Adı:** `org.hp.omen.Rgb`
- **Uygulayıcı Servis:** [[rgb-service]]

## Metotlar (Methods)

| Metot İmzası | Parametreler | Dönüş Tipi | Açıklama |
| :--- | :--- | :--- | :--- |
| `set_color(zone_val: i32, hex_color: &str)` | Bölge indeksi (`0..3`), Hex renk (`"FF00AA"`) | `String` | Belirtilen klavye bölgesinin rengini ayarlar. |
| `set_mode(mode_str: &str, speed_val: i32)` | `"static"`, `"breathing"`, `"wave"`, `"cycle"`, Hız (`1..5`) | `String` | Dinamik aydınlatma modunu ve hızını ayarlar. |
| `set_global(power_val: bool, brightness_val: i32, direction_str: &str)` | Açık/Kapalı, Parlaklık (`0..100`), Yön (`"ltr"`/`"rtl"`) | `String` | Genel parlaklık ve yön parametrelerini belirler. |
| `get_state()` | Yok | `String` (JSON) | Mevcut bölge renklerini ve aktif modu döner. |
| `set_per_key_colors(colors_json: &str)` | Tuş indeksi -> Hex eşleme JSON metni | `String` | Tuş başına aydınlatmalı modellerde tüm klavyeyi boyar. |
| `start_per_key_wizard()` | Yok | `String` | İnteraktif HID eşleme sihirbazını başlatır. |
| `light_key_index(index: u32, hex_color: &str)` | Tuş donanım indeksi, Hex renk | `String` | Sihirbaz sırasında tek bir tuşun LED'ini yakar. |
| `record_key_mapping(index: u32, key_name: &str)` | İndeks, Tuş adı (örn. `"KEY_SPACE"`) | `String` | Taranan tuşu haritaya kaydeder. |

## Renk Formatı
Hex renk değerleri kesinlikle 6 haneli onaltılık dizgi (`RRGGBB`, örneğin `"00FF88"`) formatında olmalıdır.

## İlgili Bağlantılar
- Servis Katmanı: [[rgb-service]]
- Çekirdek Sürücüsü: [[hp-omen-extra-dkms]]
- D-Bus Protokolü: [[dbus-ipc-protocol]]
