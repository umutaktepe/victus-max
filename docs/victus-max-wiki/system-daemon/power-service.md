# Güç Yönetimi ve Performans Profili Servisi

## Genel Bakış
`PowerService` (`src/victus-max-daemon/src/power.rs`), cihazın termal ve elektriksel güç sınırlarını yöneten çekirdek arka plan bileşenidir.

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

### 4. Güç Modu Profilleri (Presets) ve 30sn Emniyet Kilidi (Watchdog)
[[adr-008-power-profile-limit-presets]] ile hayata geçirilen mimari:
- **Üçlü Güç Profili:** `power-saver`, `balanced` ve `performance` modları için AMD (STAPM/Slow/Fast/Temp) ve Intel (PL1/PL2/TCC) mimarilerine göre bağımsız güç ve sıcaklık limitleri tanımlanır.
- **CPU-Tier Kalibrasyonu:** Donanım türüne göre (`AmdStandardHs`, `AmdExtremeHx`, `IntelStandard`, `IntelExtreme`) optimize fabrika limitleri sunulur.
- **30 Saniyelik Sessiz Watchdog:** HP EC/BIOS'un arka planda limitleri sıfırlamasını engellemek için aktif preset veya manuel limitler her 30 saniyede bir donanıma sessizce yeniden yazılır.
- **Slider Override Hiyerarşisi:** Manuel slider değişikliği aktif modu unselect ederek manuel override durumuna geçer; bir preset kartına tıklandığında ise preset slider'ları ezer.

### 5. Yapılandırma Kalıcılığı
Uygulanan tüm güç ayarları ve `limit_presets` yapılandırması, sistem yeniden başladığında korunmak üzere `/etc/victus-max/power.json` dosyasında saklanır (`PowerConfig`, eski `/etc/omen-space/power.json` geriye dönük fallback desteğiyle).

## Otomasyon ve Harici Entegrasyon
- **PPD Entegrasyonu:** [[power-automation-service]], GNOME/KDE güç yöneticisiyle çift yönlü senkronizasyon sağlar; PPD profili değiştiğinde ilgili güç limiti preset'i anında devreye girer.
- **Donanım Olayları:** AC şarj aletinin takılması/çıkarılması ve `systemd-logind` üzerinden uykudan uyanma (resume) olaylarında limitler gecikmesiz tazelenir.
- **Oyun Tanıma:** [[game-automation-service]], desteklenen bir oyun başladığında otomatik olarak `performance` profiline geçiş yaptırır.

## İlgili Bağlantılar
- Mimari Kararlar: [[adr-004-native-msr-and-smu-mailbox-tuning]], [[adr-007-adaptive-ryzen-power-limits]], [[adr-008-power-profile-limit-presets]]
- Donanım Sürücüsü: [[hp-wmi-driver]]
- D-Bus Sözleşmesi: [[power-dbus-interface]]
- Güç Otomasyonu: [[power-automation-service]]
- Oyun Otomasyonu: [[game-automation-service]]

