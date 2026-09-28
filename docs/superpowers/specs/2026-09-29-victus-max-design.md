# Victus Max: Akıllı Fan ve Performans Yönetim Sistemi Tasarım Belgesi

- **Tarih:** 2026-09-29
- **Durum:** Onaylandı (Spec)
- **Hedef Donanım:** HP Victus Dizüstü Bilgisayarlar (Öncelikli: Victus 16-s0xxx AMD Ryzen + NVIDIA RTX 40-serisi / Anakart 8BD4)
- **Temel Mimari:** `omen-space` Rust ekosistemi (Daemon, GUI, CLI, Tray, Overlay) + `victus-control` Better Auto Proaktif Yük Motoru ve EC Güvenlik Kuralları.

---

## 1. Genel Bakış ve Amaç

Victus Max, HP Victus ve Omen dizüstü bilgisayarlar için geliştirilmiş; `omen-space`'in zengin Linux masaüstü ekosistemini (GTK4/Libadwaita arayüzü, D-Bus IPC, sistem tepsisi, oyun içi HUD overlay, GPU TGP ve güç profilleri) ile `victus-control`'ün donanım kilitlenmelerini önleyen ve işlem yükünü proaktif takip eden **Better Auto** motorunu bir araya getiren yeni nesil bir sistem yönetim yazılımıdır.

### Temel Hedefler:
1. **Proaktif Isı Koruması:** Sıcaklığın yükselmesini beklemeden, CPU ve GPU'daki anlık iş yükü sıçramalarını yakalayıp fanları erkenden devreye sokmak (Thermal Throttling'i sıfırlamak).
2. **Akustik Konfor (Balanced Gaming):** Windows OMEN Gaming Hub'daki Balanced oyun deneyimini Linux'a getirmek. Oyunda donanımın dengeli güç tüketmesini sağlamak ve fanları 5800 RPM jet motoru devrine çıkarmadan ~3400-3900 RPM bandında sessiz ve serin (70-75°C) tutmak.
3. **Kullanıcı Ayarlı Asgari Devir (Min RPM):** Balanced modu için varsayılan minimum fan hızını **2600 RPM** olarak sabitlemek ve kullanıcının bu taban devri arayüzden dilediği gibi değiştirebilmesini sağlamak.
4. **Gömülü Denetleyici (EC) Kararlılığı:** Victus anakartlarında iki fanın aynı anda yazılmasıyla oluşan donanım kilitlenmelerini 10 saniyelik kademelendirme (stagger) ve BIOS sessiz sıfırlamalarını 90 saniyelik bekçi (watchdog) ile tamamen bertaraf etmek.
5. **Pil Tasarruflu dGPU Takibi:** NVIDIA ekran kartı runtime-suspend modundayken (uykudayken) asla `nvidia-smi` çağrısı yapmayarak harici GPU'nun uyanmasını ve pilin erimesini engellemek.

---

## 2. Sistem Mimarisi

Sistem 4 ana katmandan oluşur:

```
┌─────────────────────────────────────────────────────────────┐
│                      KULLANICI ALANI                        │
│  victus-max-gui  │  victus-max-tray  │  victus-max-overlay  │
│                     │  victus-max-cli  │                    │
└──────────────────────────────┬──────────────────────────────┘
                               │ zbus (System D-Bus)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                 victus-max-daemon (Root Servis)             │
│  ┌─────────────────────────┐   ┌─────────────────────────┐  │
│  │    Sysmon & Telemetri   │   │   Power & Platform Svc  │  │
│  │ (/proc/stat & PM-safe NV│   │ (TGP, PPAB, platform-pr)│  │
│  └────────────┬────────────┘   └────────────┬────────────┘  │
│               │                             │               │
│               ▼                             ▼               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │           🧠 Better Auto Motoru (Rust)                 │  │
│  │  - 8 Seviyeli Çift Matris (Isı & Yük)                 │  │
│  │  - Histerezis & Kademeli Düşüş Sınırlayıcı            │  │
│  │  - Profil Duyarlı Akustik Tavan (Balanced vs Perf)    │  │
│  │  - Kullanıcı Tanımlı Min RPM Tabanı (Varsayılan 2600) │  │
│  └──────────────────────────┬────────────────────────────┘  │
│                             │                               │
│                             ▼                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │            Donanım Güvenlik Katmanı                   │  │
│  │  - 10s EC Fan Stagger (Fan1 -> Bekle -> Fan2)         │  │
│  │  - 90s Watchdog & 80s Manual Reassert                 │  │
│  │  - 95°C/82°C Acil Durum Termal Kalkanı                │  │
│  └──────────────────────────┬────────────────────────────┘  │
└──────────────────────────────┼──────────────────────────────┘
                               │ sysfs / WMI
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                ÇEKİRDEK & DONANIM SÜRÜCÜSÜ                  │
│       hp-wmi (OmenCtl/Victus)  +  hp-omen-extra (RGB)       │
│    /sys/devices/platform/hp-wmi/hwmon/hwmon*/fan*_target    │
│    /sys/firmware/acpi/platform_profile                      │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Alt Sistem Detayları

### 3.1. Telemetri ve Yük Algılama Motoru (`sysmon/`)
- **CPU Yükü Hesabı:** Her 2 saniyede bir `/proc/stat` okunur. `user, nice, system, idle, iowait, irq, softirq, steal` metriklerinin önceki örneğe göre farkı (delta) alınarak net kullanım yüzdesi (`0.0 - 100.0%`) hesaplanır.
- **dGPU Takibi (Runtime-PM Safe):**
  - `/sys/bus/pci/devices/.../power/runtime_status` kontrol edilir.
  - Sadece durum `"active"` ise `nvidia-smi` çağrılarak GPU sıcaklığı ve kullanım yüzdesi (`utilization.gpu`) çekilir.
  - Durum `"suspended"` ise sorgu atlanarak dGPU uykuda bırakılır, `gpu_usage_pct = None` kabul edilir.
- **Enstantane Yapısı (`ThermalSnapshot`):**
  ```rust
  pub struct ThermalSnapshot {
      pub cpu_temp_c: Option<f64>,
      pub gpu_temp_c: Option<f64>,
      pub cpu_usage_pct: Option<f64>,
      pub gpu_usage_pct: Option<f64>,
  }
  ```

### 3.2. Better Auto Çekirdeği (`fan/better_auto.rs`)
1. **Sıcaklık Seviyesi Hesabı (Histerezisli):**
   - Yükselme Eşikleri: `[45.0, 54.0, 62.0, 68.0, 73.0, 78.0, 83.0°C]`
   - Düşme Eşikleri: `[42.0, 51.0, 59.0, 65.0, 69.5, 74.5, 79.0°C]`
2. **Yük Seviyesi Hesabı (Proaktif):**
   - Yük Eşikleri: `[25.0, 35.0, 48.0, 58.0, 66.0, 74.0, 82.0%]`
   - Hedef Seviye: `target_level = max(temp_level, usage_level)`.
3. **Akustik Dalgalanma ve Soğuma Koruması:**
   - Düşüş Freni: Seviye tek seferde en fazla 1 kademe düşebilir (`target_level = max(target_level, prev_level - 1)`).
   - Cooldown Kalkanı: Seviye 7-8 görüldüyse en az 30 saniye, Seviye 5-6 görüldüyse en az 15 saniye taban seviye korunur.
4. **Performans Moduna Duyarlı Akustik Tavan (Context-Aware):**
   - **Eco Modu:** Maksimum Seviye 3 (~3100 RPM) ile sınırlandırılır.
   - **Balanced Modu:** Akustik tavan Seviye 5 (~3900 RPM) olarak uygulanır. Oyun yükünde bile 5800 RPM'e çıkmaz; sıcaklık 88°C'yi aşmadıkça fısıltı seviyesinde kalır.
   - **Performance Modu:** Tüm tavanlar kalkar, fanlar Seviye 8'e kadar hızlanarak maksimum soğutma sağlar.

### 3.3. Dinamik ve Kullanıcı Ayarlı Asgari Devir (Min RPM) ve Akustik Tavan
- Yapılandırma dosyası (`/etc/victus-max/fan.json` veya `/etc/omen-space/fan.json`) içinde saklanır:
  ```json
  {
      "min_fan_rpm": 2600,
      "balanced_min_rpm": 2600,
      "eco_min_rpm": 2400,
      "acoustic_ceiling_level": 5,
      "acoustic_ceiling_rpm": 3900
  }
  ```
- **RPM Enterpolasyonu:**
  - Taban: `min_fan_rpm` (Varsayılan: 2600 RPM)
  - Tavan: Donanım maksimum devri (`fan1_max` ~5800 RPM, `fan2_max` ~6100 RPM)
  - Adım büyüklüğü: `(max_rpm - min_rpm) / (8 - 1)`
  - Her seviye için tam devir: `rpm = min_rpm + (level - 1) * step`
- **Kullanıcı Ayarlı Akustik Tavan (Acoustic Ceiling):**
  - Balanced modunda varsayılan Seviye 5 (~3900 RPM) olan tavan, kullanıcının tercihine göre Seviye 3 (Süper Sessiz, ~3100 RPM) ile Seviye 8 (Tam Açık, ~5800+ RPM) arasında ayarlanabilir.
  - Acil Durum Baypası: Sıcaklık kritik eşiği (>88°C) aşarsa donanım güvenliği için tavan geçici olarak delinir, sıcaklık normale dönünce tavan tekrar kilitlenir.
- **Arayüz Kontrolü:** GUI Ayarlar veya Fan sekmesinde kullanıcının hem asgari devri (2000 - 3500 RPM) hem de akustik tavanı (Seviye 3-8 / 3100-5800 RPM) kaydırıcı/spinbox ile kolayca ayarlayabilmesi sağlanır.

### 3.4. Donanım Zamanlaması ve Bekçi (Hardware Stagger & Watchdog)
- **10 Saniyelik Stagger:** Fan 1 hızı yazıldıktan sonra, Fan 2 yazılmadan önce asenkron `tokio::time::sleep(Duration::from_secs(10))` beklenir.
- **90 Saniyelik Watchdog:** Isı veya yük değişmese dahi 90 saniyede bir mevcut hedefler sysfs'e yeniden basılır.
- **80 Saniyelik Manual Reassert:** HP BIOS'un otomatik moda dönmesini önlemek için periyodik `pwm1_enable = 1` yazılır.

---

## 4. D-Bus ve Arayüz Değişiklikleri

### 4.1. D-Bus Arayüzü (`org.hp.omen.Fan` / `org.hp.victusmax.Fan`)
- `SetFanMode("better_auto")`: Better Auto modunu aktif eder.
- `GetMinFanRpm() -> u32`: Mevcut taban devri döner (örn. 2600).
- `SetMinFanRpm(rpm: u32) -> bool`: Taban devri günceller ve kaydeder.
- `GetAcousticCeiling() -> u32`: Mevcut akustik tavan seviyesini döner (örn. 5).
- `SetAcousticCeiling(level: u32) -> bool`: Akustik tavan seviyesini günceller ve kaydeder.

### 4.2. GUI (GTK4 / Libadwaita) Değişiklikleri
- **Fan Modları Alanı:**
  - `Auto` kartının yanına **🧠 Better Auto (Smart Load)** kartı eklenir.
  - Açıklama altlığı: *"Workload-aware proactive cooling"*.
- **Ayarlar Sekmesi / Fan Kartı:**
  - *"Asgari Fan Devri (Minimum Fan RPM)"* ayar alanı: Kullanıcının 2000 - 3500 RPM arasında 100'er RPM adımlarla seçim yapabileceği AdwSpinRow / Slider.
  - *"Akustik Tavan (Acoustic Ceiling)"* ayar alanı: Balanced modu için maksimum izin verilen ses/devir kademesini (Seviye 3 - 8 arası) seçebileceği ayar kutusu.

---

## 5. Doğrulama ve Test Kriterleri

1. **Yük Testi:** `stress-ng --cpu 8` veya bir oyun başlatıldığında, sıcaklık henüz 50°C iken fanların 2 saniye içinde Seviye 5-6 devirlerine fırladığının loglardan doğrulanması.
2. **Akustik Tavan Testi:** Balanced modunda oyun oynanırken fan devrinin 4000 RPM'i aşmadığının ve sıcaklığın 75°C civarında sabit kaldığının gözlemlenmesi.
3. **Min RPM Testi:** Ayarlardan asgari devir 2800 veya 2400 RPM yapıldığında boştaki fan devrinin bu değere kilitlendiğinin teyidi.
4. **Stagger ve Watchdog Testi:** `dmesg` ve journal loglarında hiçbir EC zaman aşımı veya fan kilitlenmesi hatası oluşmadığının doğrulanması.
