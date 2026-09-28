# ADR-003: Çakışmasız Yoldaş (Companion) DKMS Sürücü Mimarisi

## Durum
Kabul Edildi

## Tarih
2024-08-10 (Revizyon: 2026-09-29)

## Bağlam
HP Omen ve Victus dizüstü bilgisayarlarında klavye aydınlatması; statik arka ışık, 4 bölgeli (4-zone) RGB veya tuş başına (per-key) RGB olmak üzere farklı donanım mimarileriyle sunulmaktadır.

Mainline Linux çekirdeğinde yer alan resmi `hp-wmi` sürücüsü; temel ACPI olaylarını, uçak modu anahtarını (rfkill), sıcaklık profillerini ve donanım kısayol tuşlarını yönetmektedir. Ancak mainline `hp-wmi`, HP Omen klavyelerindeki RGB aydınlatma bölge renklerini (`HPWMI_BACKLIGHT`, `HPWMI_COLOR_SET_QUERY`) programlamak için gerekli olan sysfs arayüzlerine sahip değildir.

Linux çekirdeğinde aynı WMI GUID'e (`5FB7F034-2C63-45E9-BE91-3D44E2C707E4`) iki farklı sürücü modülü `MODULE_ALIAS` ile bağlanmaya çalıştığında çekirdek aygıt çakışması (device collision / probe failure) meydana gelmekte ve stock `hp-wmi` sürücüsü devre dışı kalabilmektedir.

## Alternatifler
- **Alternatif A (Stock hp-wmi Sürücüsünü Yamalamak ve Değiştirmek):** Dağıtımın yerleşik `hp-wmi.ko` modülünü ezerek yamalı özel bir sürüm yüklemek. (Her çekirdek güncellemesinde bozulması, DKMS derleme uyumsuzlukları ve sistem stabilitesi riskleri nedeniyle reddedildi).
- **Alternatif B (Kullanıcı Alanından Ham HIDRaw Yazımı):** Klavye USB HID arayüzü üzerinden kullanıcı alanından ham bayt göndermek. (Omen klavyelerinin WMI/ACPI tabanlı olması ve USB HID uç noktası sunmaması nedeniyle uygulanamaz).
- **Alternatif C (WMI GUID Talep Etmeyen Companion Sürücü):** Stock `hp-wmi` ile birlikte çalışabilen, hiçbir WMI GUID'i rezerve etmeyen (`MODULE_ALIAS` içermeyen) bağımsız bir DKMS modülü (`hp-omen-extra`) geliştirmek.

## Karar
Sistem [[hp-omen-extra-dkms]] (`driver/hp-omen-extra.c`) adında özel bir **Yoldaş (Companion) Kernel Modülü** mimarisini benimsemiştir:
1. **WMI GUID Paylaşımı (Non-Exclusive Probe):** Sürücü içerisinde kasıtlı olarak hiçbir `MODULE_ALIAS("wmi:...")` tanımlanmamıştır. Bu sayede Linux çekirdeği WMI aygıtını stock `hp-wmi` sürücüsüne tahsis ederken çakışma yaşanmaz.
2. **Bağımsız ACPI Yürütme:** `hp-omen-extra`, HP BIOS WMI nesnesine doğrudan ACPI değerlendirmesi (`acpi_evaluate_object`) ve WMI blok sorguları (`wmi_evaluate_method`) üzerinden bağımsız ve atomik olarak erişir.
3. **Standart Sysfs Soyutlaması:** Sürücü, sistemde `/sys/devices/platform/hp-omen-extra/` altında `zone0`..`zone7` renk ve parlaklık dosyalarını açar.
4. **DKMS Entegrasyonu:** [[packaging-and-dkms]] yapılandırması sayesinde Linux çekirdeği her güncellendiğinde sürücü otomatik olarak arka planda derlenir.

## Sonuçlar
### Olumlu
- **Tam Birlikte Yaşama (Zero Clash):** Resmi `hp-wmi` sürücüsünün yönettiği termal profiller, wifi düğmesi ve fn tuşları hiçbir kesintiye uğramadan çalışmaya devam eder.
- **Kesintisiz RGB Kontrolü:** [[rgb-service]] katmanı, karmaşık ACPI/WMI bayt dizilimleriyle uğraşmak yerine sürücünün sağladığı temiz sysfs arayüzüne yazarak RGB renklerini ayarlar.
- **Güvenli Çekirdek Güncellemeleri:** Kullanıcının dağıtım çekirdeğini güncellemesi durumunda DKMS sistemi sürücüyü hatasız bir biçimde yeniden bağlar.

### Olumsuz / Trade-off
- Kullanıcının sisteminde `dkms` paketinin ve çekirdek başlıklarının (`linux-headers`) kurulu olması bir zorunluluktur.
- Bazı çok yeni TUF/Victus karma anakartlarda WMI arabelleği (datasize) farklılıkları sürücüde model bazlı istisnalar gerektirebilir.

## İlgili Bağlantılar
- Donanım Sürücüsü: [[hp-omen-extra-dkms]]
- Çekirdek Sürücüsü: [[hp-wmi-driver]]
- Servis Katmanı: [[rgb-service]]
- Paketleme ve DKMS: [[packaging-and-dkms]]
