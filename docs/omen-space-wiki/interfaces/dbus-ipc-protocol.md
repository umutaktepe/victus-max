# D-Bus Süreçler Arası İletişim Protokolü (IPC)

## Genel Bakış
OMEN Space, Linux sistem mimarisinde ayrıcalık izolasyonunu sağlamak amacıyla D-Bus (Desktop Bus) IPC protokolünü kullanır.

Tüm D-Bus arayüz tanımları, tipleri ve proxy makroları `omen-types` kütüphanesinde (`src/omen-types/src/lib.rs`) toplanmıştır. Bu sayede hem sunucu tarafı (`omen-space-daemon`) hem de istemciler (`omen-gui`, `omen-cli`, `omen-overlay`, `omen-tray`) aynı tip tanımlarını ve arayüz sözleşmelerini derleme anında doğrular.

## D-Bus Ad Alanı ve Nesne Yolu Hiyerarşisi

- **Veri Yolu (Bus):** Linux System Bus (`system_bus`), yetkisiz test ortamlarında Session Bus (`session_bus`) fallback.
- **Hedef Servis Adı (Well-Known Name):** `org.hp.omen`
- **Tersine Uyumluluk Adı:** `com.yyl.hpmanager.*` (Eski Python OmenCtl istemcileri için).

### Nesne Yolu ve Arayüz Haritası

```
/org/hp/omen
├── Fan           -> org.hp.omen.Fan
├── Power         -> org.hp.omen.Power
├── Rgb           -> org.hp.omen.Rgb
├── Mux           -> org.hp.omen.Mux
├── Platform      -> org.hp.omen.Platform
├── Undervolt     -> org.hp.omen.Undervolt
├── Ryzen         -> org.hp.omen.Ryzen
├── SysMon        -> org.hp.omen.SysMon
└── AppProfiles   -> org.hp.omen.AppProfiles
```

## Proxy Makroları (`zbus::proxy`)
İstemciler doğrudan düşük seviyeli D-Bus baytlarıyla uğraşmak yerine `omen-types` tarafından sağlanan Rust trait proxy'lerini çağırır:

```rust
#[proxy(
    interface = "org.hp.omen.Power",
    default_service = "org.hp.omen",
    default_path = "/org/hp/omen/Power"
)]
pub trait Power {
    async fn set_power_profile(&self, profile: &str) -> zbus::Result<String>;
    async fn get_power_profile(&self) -> zbus::Result<String>;
    async fn set_power_limits(&self, enabled: bool, pl1: i32, pl2: i32) -> zbus::Result<String>;
}
```

## Sinyal Akışı (Signals Architecture)
- **`telemetry_updated(json_stats)`:** [[sysmon-service]] tarafından her 3 saniyede bir yayınlanır. İstemciler veri yoklama (polling) yapmadan dinler.
- **`thermal_protection_alert(active)`:** [[fan-service]] acil durum termal koruması tetiklendiğinde yayınlanır.
- **`macro_key_pressed(key_name)`:** [[platform-service]] tarafından yakalanan donanım tuşlarını yayınlar.

## İlgili Bağlantılar
- Mimari Karar: [[adr-001-rust-daemon-client-split]]
- Güvenlik Kuralları: [[polkit-dbus-security]]
- Arayüz Kontratları: [[fan-dbus-interface]], [[power-dbus-interface]], [[sysmon-dbus-interface]]
