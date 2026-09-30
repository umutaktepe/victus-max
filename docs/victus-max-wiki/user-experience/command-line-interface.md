# Komut Satırı Arayüzü (CLI & Scripting)

## Genel Bakış
`victus-max-cli` (`src/victus-max-cli/`), terminal tutkunları ve otomasyon betikleri için tasarlanmış tam yetenekli, hızlı bir komut satırı arayüzüdür.

CLI, D-Bus üzerinden [[daemon-overview]] ile konuşarak grafik arayüzde ([[gui-application]]) yapılabilen tüm işlemleri saniyeler içinde terminalden gerçekleştirir.

## Komut Yapısı ve Modüller

```
victus-max-cli [ALT_KOMUT] [SEÇENEKLER]
```

### 1. Fan Komutları (`commands/fan.rs`)
- `victus-max-cli fan mode [auto|max|custom]` : Fan çalışma modunu değiştirir.
- `victus-max-cli fan set <yüzde>` : Manuel fan yüzdesi belirler (örn: `victus-max-cli fan set 70`).
- `victus-max-cli fan info` : Anlık fan devirlerini ve sıcaklık durumunu döner.

### 2. Güç Komutları (`commands/power.rs`)
- `victus-max-cli power profile [power-saver|balanced|performance]` : Termal profili ayarlar.
- `victus-max-cli power limits --pl1 <watt> --pl2 <watt>` : Intel RAPL güç limitlerini uygular.

### 3. RGB Komutları (`commands/rgb.rs`)
- `victus-max-cli rgb color <bölge_no> <hex_kod>` : Belirli bir bölgenin rengini boyar (örn: `victus-max-cli rgb color 0 FF0055`).
- `victus-max-cli rgb mode <mod_adı> [hız]` : Dalga, nefes alma veya gökkuşağı efektini başlatır.

### 4. Sistem ve Teşhis Komutları (`commands/system.rs` & `fetch.rs`)
- `victus-max-cli fetch` : Victus/OMEN donanımını, BIOS sürümünü ve telemetrisini renkli ASCII logosuyla ekrana basan Neofetch/Fastfetch benzeri çıktı.
- `victus-max-cli triage` : Hata ayıklama günlüğü ve donanım dökümünü (`triage bundle`) üretir.

### 5. Overlay Denetimi (`commands/overlay.rs`)
- `victus-max-cli overlay toggle` : Oyun içi HUD penceresini ([[quick-hud-overlay]]) görünür yapar veya gizler.

## İlgili Bağlantılar
- D-Bus Protokolü: [[dbus-ipc-protocol]]
- Çekirdek Servis: [[daemon-overview]]
- Hızlı HUD: [[quick-hud-overlay]]
