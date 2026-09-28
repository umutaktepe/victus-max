# Sistem Telemetrisi ve Donanım Şartnamesi (Telemetry Spec)

## Genel Bakış
OMEN Space istemcileri ile çekirdek daemon arasındaki veri akışı, `omen-types` kütüphanesinde tanımlanan strongly-typed Rust struct yapılarıyla yürütülür.

Bu yapılar, D-Bus sinyali (`telemetry_updated`) ve `get_hardware_specs()` metotlarında JSON formatında taşınır.

## `SystemStats` Veri Modeli

`SystemStats`, cihazın dinamik durumunu temsil eden 3 saniyelik periyodik telemetri paketidir:

| Alan Adı | Tip | Birim | Açıklama |
| :--- | :--- | :--- | :--- |
| `cpu_temp` | `i32` | °C | İşlemci paket tepe sıcaklığı |
| `cpu_load` | `f64` | % | CPU çekirdek kullanım yüzdesi (`0.0 - 100.0`) |
| `cpu_pwr` | `f64` | Watt | CPU anlık paket güç çekimi |
| `fan_rpm` | `i32` | RPM | Ortalama/Birincil fan dönüş hızı |
| `fan1_rpm` | `i32` | RPM | 1. Fan (CPU) devir sayısı |
| `fan2_rpm` | `i32` | RPM | 2. Fan (GPU) devir sayısı |
| `gpu_temp` | `i32` | °C | Ayrık NVIDIA veya AMD GPU sıcaklığı |
| `gpu_load` | `f64` | % | GPU grafik motoru kullanım yüzdesi |
| `gpu_pwr` | `f64` | Watt | GPU anlık güç tüketimi |
| `ram_used_gb` | `f64` | GB | Kullanılan fiziksel bellek |
| `ram_total_gb`| `f64` | GB | Toplam kurulu fiziksel bellek |
| `ram_frac` | `f64` | Oran | RAM doluluk oranı (`0.0 - 1.0`) |
| `disk_used_gb`| `f64` | GB | Kök dosya sistemi kullanılan alan |
| `disk_total_gb`| `f64` | GB | Kök dosya sistemi toplam alan |
| `disk_frac` | `f64` | Oran | Disk doluluk oranı (`0.0 - 1.0`) |
| `total_pwr` | `f64` | Watt | CPU + GPU toplam çekilen güç |
| `cpu_throttle_count` | `u32` | Adet | Sistem açılışından beri oluşan termal kısma sayısı |
| `chassis_temp`| `i32` | °C | EC tarafından raporlanan kasa içi ortam sıcaklığı |
| `board_verified` | `bool`| Bayrak | Anakartın `boards.json` matrisinde doğrulanmış olup olmadığı |

## `HardwareSpecs` Statik Donanım Modeli

Cihazın sistem kimliğini belirten statik özellik dökümü:
```rust
pub struct HardwareSpecs {
    pub product_name: String,   // Örn: "OMEN by HP Laptop 16-k0xxx"
    pub cpu_spec: String,       // Örn: "12th Gen Intel Core i7-12700H"
    pub gpu_spec: String,       // Örn: "NVIDIA GeForce RTX 3070 Ti Laptop GPU"
    pub ram_spec: String,       // Örn: "32 GB DDR5 4800 MHz"
    pub ssd_spec: String,       // Örn: "1024 GB NVMe PCIe Gen4"
    pub os_spec: String,        // Örn: "Arch Linux (Rolling)"
    pub bios_version: String,   // Örn: "F.17"
    pub ec_version: String,     // Örn: "38.25"
    pub vbios_version: String,  // Örn: "94.04.7E.00.8C"
    pub nvidia_driver: String,  // Örn: "560.35.03"
    pub kernel_version: String, // Örn: "6.11.2-arch1-1"
}
```

## İlgili Bağlantılar
- Servis Katmanı: [[sysmon-service]]
- D-Bus Arayüzü: [[sysmon-dbus-interface]]
- Görsel Tüketici: [[quick-hud-overlay]], [[gui-application]]
