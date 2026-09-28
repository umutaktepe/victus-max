# HP OMEN Extra DKMS Yoldaş Sürücüsü

## Genel Bakış
`hp-omen-extra`, HP Omen ve Victus serisi taşınabilir bilgisayarların RGB klavye arka aydınlatmasını yönetmek amacıyla geliştirilmiş açık kaynaklı bir Linux çekirdek modülüdür (`driver/hp-omen-extra.c`).

Stock Linux çekirdeğindeki `hp-wmi` sürücüsünün RGB kontrol kabiliyeti barındırmaması nedeniyle, sistem çekirdeğini yamalamadan veya çakışma yaratmadan donanım düzeyinde RGB desteği sunmak üzere tasarlanmıştır.

## Çekirdek İçi Tasarım ve WMI Entegrasyonu

### 1. GUID Paylaşımı ve Sürücü Bir Arada Yaşama
Standart Linux modülleri belirli WMI GUID'leri sahiplenmek için `MODULE_ALIAS("wmi:...")` makrosunu kullanır. Birden fazla sürücü aynı GUID'i talep ettiğinde Linux aygıt modeli çakışma (device conflict) yaşar.

`hp-omen-extra`, bu çakışmayı önlemek için hiçbir GUID sahiplenmesi yapmaz:
- **Hedef BIOS GUID:** `5FB7F034-2C63-45E9-BE91-3D44E2C707E4` (`HPWMI_BIOS_GUID`)
- **Yöntem:** GUID çekirdeğin WMI altyapısında kayıtlıyken, ACPI yöntem çağrıları doğrudan `wmi_evaluate_method` fonksiyonu üzerinden bağımsız olarak yürütülür.
- **Tasarım Gerekçesi:** Detaylı mimari analiz için bkz: [[adr-003-companion-dkms-driver-non-clashing]].

### 2. Komut ve Veri Yapısı (WMI Command Types)
Sürücü, BIOS'a gönderilecek verileri `bios_args` yapısıyla paketler:
```c
struct bios_args {
    u32 signature;    // 0x55434553 ("SECU")
    u32 command;      // HPWMI_BACKLIGHT (0x20009)
    u32 commandtype;  // HPWMI_COLOR_SET_QUERY (0x03) veya BRIGHTNESS (0x05)
    u32 datasize;     // Gönderilen bayt boyutu
    u8  data[];       // Renk verisi (RGB baytları)
};
```

### 3. Sysfs Arayüzü Soyutlaması
Sürücü yüklendiğinde platform aygıtı olarak `/sys/devices/platform/hp-omen-extra/` dizinini oluşturur:
- `zone0` - `zone3`: 4 bölgeli klavyeler için her bir bölgenin 24-bit Hex RGB değeri (Örn: `FF0000`).
- `zone4` - `zone7`: 8 bölgeli veya genişletilmiş LED şeritlerine sahip modeller için ayrılmış ek bölgeler.
- `brightness`: 0-100 arası klavye parlaklık seviyesi.

## Kullanıcı Alanı Entegrasyonu
[[rgb-service]] katmanı doğrudan bu sysfs dosyalarına asenkron olarak yazar. Statik renkler, dalga (wave), nefes alma (breathing) ve gökkuşağı (rainbow) gibi yazılımsal animasyonlar, daemon içindeki timer döngüleri tarafından hesaplanıp buraya basılır.

## Dağıtım ve DKMS
Modül, Dinamik Çekirdek Modülü Desteği ([[packaging-and-dkms]]) ile sisteme entegre edilir. `driver/dkms.conf` yapılandırması sayesinde kernel güncellemelerinde otomatik olarak yeniden derlenir.

## İlgili Bağlantılar
- Mimari Karar: [[adr-003-companion-dkms-driver-non-clashing]]
- Donanım Sürücüsü: [[hp-wmi-driver]]
- Servis Katmanı: [[rgb-service]]
- Paketleme: [[packaging-and-dkms]]
