# Komut Satırı Arayüzü (CLI & Scripting)

## Genel Bakış
`omen-cli` (`src/omen-cli/`), terminal tutkunları ve otomasyon betikleri için tasarlanmış tam yetenekli, hızlı bir komut satırı arayüzüdür.

CLI, D-Bus üzerinden [[daemon-overview]] ile konuşarak grafik arayüzde ([[gui-application]]) yapılabilen tüm işlemleri saniyeler içinde terminalden gerçekleştirir.

## Komut Yapısı ve Modüller

```
omen-cli [ALT_KOMUT] [SEÇENEKLER]
```

### 1. Fan Komutları (`commands/fan.rs`)
- `omen-cli fan mode [auto|max|custom]` : Fan çalışma modunu değiştirir.
- `omen-cli fan set <yüzde>` : Manuel fan yüzdesi belirler (örn: `omen-cli fan set 70`).
- `omen-cli fan info` : Anlık fan devirlerini ve sıcaklık durumunu döner.

### 2. Güç Komutları (`commands/power.rs`)
- `omen-cli power profile [power-saver|balanced|performance]` : Termal profili ayarlar.
- `omen-cli power limits --pl1 <watt> --pl2 <watt>` : Intel RAPL güç limitlerini uygular.

### 3. RGB Komutları (`commands/rgb.rs`)
- `omen-cli rgb color <bölge_no> <hex_kod>` : Belirli bir bölgenin rengini boyar (örn: `omen-cli rgb color 0 FF0055`).
- `omen-cli rgb mode <mod_adı> [hız]` : Dalga, nefes alma veya gökkuşağı efektini başlatır.

### 4. Sistem ve Teşhis Komutları (`commands/system.rs` & `fetch.rs`)
- `omen-cli fetch` : OMEN/Victus donanımını, BIOS sürümünü ve telemetrisini renkli ASCII logosuyla ekrana basan Neofetch/Fastfetch benzeri çıktı.
- `omen-cli triage` : Hata ayıklama günlüğü ve donanım dökümünü (`triage bundle`) üretir.

### 5. Overlay Denetimi (`commands/overlay.rs`)
- `omen-cli overlay toggle` : Oyun içi HUD penceresini ([[quick-hud-overlay]]) görünür yapar veya gizler.

## İlgili Bağlantılar
- D-Bus Protokolü: [[dbus-ipc-protocol]]
- Çekirdek Servis: [[daemon-overview]]
- Hızlı HUD: [[quick-hud-overlay]]
