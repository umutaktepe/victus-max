# Güç Yönetimi ve Performans Profili Servisi

## Genel Bakış
`PowerService` (`src/omen-space-daemon/src/power.rs`), cihazın termal ve elektriksel güç sınırlarını yöneten çekirdek arka plan bileşenidir.

Servis, ACPI termal profillerini, Intel RAPL (Running Average Power Limit) paket sınırlarını ve NVIDIA GPU TGP güç bütçelerini tek bir çatı altında koordine eder. D-Bus üzerinde `org.hp.omen.Power` arayüzünü ([[power-dbus-interface]]) dinler.

## Temel Görevler ve Donanım Entegrasyonu

### 1. ACPI Termal Profilleri
BIOS seviyesinde önceden tanımlanmış güç ve fan stratejilerini uygular:
- **`power-saver` / `quiet`:** Düşük voltaj, minimum fan gürültüsü ve kısıtlanmış saat hızları.
- **`balanced`:** Günlük kullanım için optimize edilmiş dinamik güç ölçekleme.
- **`performance`:** Donanımın izin verdiği en yüksek saat hızları, agresif fan eğrisi ve maksimum TDP.
- **Uygulama:** Bu profiller [[hp-wmi-driver]] aracılığıyla `/sys/devices/platform/hp-wmi/thermal_profile` düğümüne yazılır.

### 2. Intel RAPL Güç Limitleri (PL1 / PL2)
İşlemcinin sürdürülebilir (PL1) ve anlık patlama (PL2) güç tüketimini Watt cinsinden sınırlar:
- **Sysfs Arabirimi:** `/sys/class/powercap/intel-rapl/intel-rapl:0/`
  - `constraint_0_power_limit_uw` (PL1 - Uzun vadeli sınır, mikrowatt cinsinden).
  - `constraint_1_power_limit_uw` (PL2 - Kısa vadeli tepe güç sınırı).
- Servis, kullanıcının belirlediği Watt değerini mikrowatt'a (`W * 1_000_000`) çevirerek atomik olarak sysfs'e yazar.

### 3. GPU TGP (Total Graphics Power) Yönetimi
Ayrık NVIDIA ekran kartı bulunan sistemlerde, pil tüketimini düşürmek veya aşırı ısınmayı engellemek için `nvidia-smi` üzerinden güç tavanı ayarlanır.

### 4. Yapılandırma Kalıcılığı
Uygulanan tüm güç ayarları, sistem yeniden başladığında korunmak üzere `/etc/omen-space/power.json` dosyasında saklanır (`PowerConfig`).

## Otomasyon ve Harici Entegrasyon
- **PPD Entegrasyonu:** [[power-automation-service]], GNOME güç yöneticisiyle çift yönlü senkronizasyon sağlar.
- **Oyun Tanıma:** [[game-automation-service]], desteklenen bir oyun başladığında otomatik olarak `performance` profiline geçiş yaptırır.

## İlgili Bağlantılar
- Donanım Sürücüsü: [[hp-wmi-driver]]
- D-Bus Sözleşmesi: [[power-dbus-interface]]
- Güç Otomasyonu: [[power-automation-service]]
- Oyun Otomasyonu: [[game-automation-service]]
