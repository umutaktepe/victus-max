# ADR-008: Güç Modu Profilleri (Presets), 30sn Emniyet Kilidi ve Donanımsal Koruma Mimarisi

- **Tarih:** 2026-10-13
- **Durum:** Kabul Edildi / Uygulandı
- **Karar Vericiler:** Victus Max Çekirdek Ekibi
- **İlgili Kararlar:** [[adr-004-native-msr-and-smu-mailbox-tuning]], [[adr-007-adaptive-ryzen-power-limits]]

---

## 1. Bağlam ve Problem (Context)

HP Victus ve OMEN dizüstü bilgisayarlarında çalışan HP EC (Embedded Controller) ve BIOS yazılımları, belirli aralıklarla veya AC şarj aletinin takılması/çıkarılması, sistemin uykudan uyanması (suspend/hibernate) gibi donanım kesmelerinde işlemcinin güç limitlerini (AMD Ryzen SMU STAPM/PPT, Intel RAPL PL1/PL2 ve TCC/Tctl sıcaklık sınırlarını) sessizce fabrika ayarlarına sıfırlamaktadır.

Kullanıcılar daha önce bu donanım davranışını aşmak için harici kabuk betikleri (`apply-ryzen-limits.sh`, `watch-power-profile.sh`), systemd zamanlayıcıları (`ryzen-limits.timer` her 30 saniyede bir tetiklenen oneshot servis) ve udev kuralları (`99-ryzen-power.rules`) kullanmaktaydı. Bu durum şu sorunlara yol açmaktaydı:
1. Dağınık yapılandırma dosyaları ve kırılgan bash betiklerine bağımlılık.
2. Yalnızca AMD platformuna odaklanıp Intel RAPL/MSR mimarisini dışarıda bırakma.
3. Masaüstü ortamı güç profilleri (`power-profiles-daemon`, GNOME/KDE paneli) ile Victus Max arayüzü arasında çift yönlü senkronizasyon eksikliği.
4. Gelişmiş güç ayarlarının sistem yeniden başlatıldığında korunamaması.

---

## 2. Değerlendirilen Alternatifler (Alternatives)

### Alternatif 1: Harici Systemd Servisleri ve Betik Yönetimi
- Daemon'ın harici bash/sh betiklerini çalıştırması ve systemd timer birimlerini yönetmesi.
- *Neden Reddedildi:* Platform bağımlılıklarını artırır, hata ayıklamayı zorlaştırır, IPC yerine dosya/alt-süreç çağırma maliyeti yaratır ve Intel mimarisiyle homojen bir yapı sunamaz.

### Alternatif 2: Yalnızca Manuel Slider Kontrolü
- Kullanıcıya hazır profil tanımlamadan sadece slider'lar sunmak.
- *Neden Reddedildi:* Her güç modu değişiminde (örn. pilde çalışırken veya oyuna girildiğinde) kullanıcının manuel olarak her slider'ı tekrar ayarlamasını gerektirir.

### Alternatif 3: Yerel Rust Mimarisi, CPU-Tier Fabrika Varsayılanları ve Sessiz Watchdog (Seçilen)
- Tüm profil durum yönetiminin `/etc/victus-max/power.json` altında `limit_presets` ile tutulması.
- `cpu_tier.rs` üzerinden CPU mimarisine (AMD HS, HX, U; Intel H, HX) göre kalibre edilmiş fabrika varsayılanlarının sunulması.
- 30 saniyelik arka plan emniyet kilidinin (`PowerService` Tokio task) ve `systemd-logind` / AC tetikleyicilerinin (`power_automation.rs`) yerel Rust asenkron döngüleriyle yürütülmesi.

---

## 3. Alınan Karar (Decision)

1. **Preset Veri Yapısı (`omen_types`):**
   - `PowerModeLimits` (STAPM/PL1, Slow PPT, Fast PPT/PL2, Tctl/TCC Temp) ve `PowerModeProfilesConfig` (Power Save, Balanced, Performance) tanımlandı.
2. **D-Bus IPC Sözleşmesi (`org.hp.omen.Power`):**
   - `GetPowerPresets() -> JSON`: Aktif mod, manuel override durumu, profiller ve CPU-tier varsayılanlarını döner.
   - `SetPowerPresetMode(mode: String)`: Belirtilen modu aktif eder, donanıma yazar ve ACPI platform profilini eşitler.
   - `SavePowerPreset(mode: String, ...)`: Kullanıcının belirlediği özel limitleri kaydeder.
   - `ResetPowerPresets()`: CPU-tier fabrika ayarlarına geri döndürür.
3. **30 Saniyelik Sessiz Emniyet Kilidi (Silent Watchdog):**
   - `PowerService::new()` içinde 30 saniyelik sessiz bir tokio zamanlayıcısı başlatıldı. Aktif modun limitlerini sessizce yenileyerek BIOS/EC sıfırlamalarını bertaraf eder. Masaüstü bildirimi göndermez.
4. **Olay Tabanlı Donanım Tetikleyicileri (`power_automation.rs`):**
   - PPD `ActiveProfile` değişimi anında ilgili preset modunu tetikler.
   - AC şarj aleti takıldığında `performance`, pilde çalışmaya geçildiğinde `power-saver` moduna otomatik geçiş yapılır.
   - `org.freedesktop.login1` üzerinden `PrepareForSleep(false)` sinyali yakalanarak sistem uykudan (suspend/hibernate) uyandığında 1.5 sn sonra donanım limitleri otomatik yenilenir.
5. **Slider ve Preset Override Hiyerarşisi:**
   - 3 moddan biri seçiliyken Advanced Power slider'ları değiştirilip "Apply & Save" dendiğinde aktif mod sıfırlanır ve sistem **"Manuel Override"** moduna geçer.
   - 3 moddan birine tıklandığında ise mod slider'ları ezer ve slider'lar o modun kayıtlı değerlerine güncellenir.
6. **Kullanıcı Arayüzü & CLI:**
   - GTK4 / Libadwaita arayüzünde (`undervolt.rs`) 3 adet mod kartı ve mimariye duyarlı (Ryzen: STAPM/Slow/Fast/Temp; Intel: PL1/PL2/TCC) modal düzenleme penceresi eklendi.
   - `victus-max-cli power presets` ve `victus-max-cli power limits` komutları terminalden izleme ve yönetim imkanı sundu.

---

## 4. Sonuçlar ve Değerlendirme (Consequences)

### Olumlu Sonuçlar (Pros):
- **Sıfır Dış Bağımlılık:** Dağınık bash scriptleri, harici timer servisleri veya udev kurallarına gerek kalmadı.
- **Evrensel Uyumluluk:** Hem AMD Ryzen (SMU/RyzenAdj) hem de Intel (RAPL sysfs + TCC MSR) donanımları tek tip arayüzle yönetilmektedir.
- **Kalıcılık:** Bilgisayar yeniden başlatıldığında, uykudan uyandığında veya adaptör takılıp çıkarıldığında son seçilen güç profili garanti altına alınır.
- **PPD ve Masaüstü Entegrasyonu:** GNOME/KDE pil menüsünden yapılan mod değişiklikleri ile Victus Max tam senkronize çalışır.

### Olumsuzluklar ve Ödünleşimler (Trade-offs):
- Her 30 saniyede bir arka plan döngüsü çalışmaktadır; ancak `RYZENADJ_MUTEX` ile serileştirildiği ve donanıma yalnızca yazmaç güncellemesi yapıldığı için CPU maliyeti ölçülemeyecek kadar düşüktür (<0.01% CPU).

---

## 5. İlgili Bağlantılar (Wikilinks)

- [[adr-004-native-msr-and-smu-mailbox-tuning]] — MSR ve SMU posta kutusu serileştirme mimarisi.
- [[adr-007-adaptive-ryzen-power-limits]] — AMD Ryzen nesillerine duyarlı güç limitleri.
- [[power-service]] — D-Bus güç yönetim servisi.
- [[undervolt-service]] — İşlemci voltaj ve sıcaklık ofset kontrolü.
- [[quick-hud-overlay]] — Hızlı kontrol arayüzü.
