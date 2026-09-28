# Sıfır Gecikmeli Donanım Kısayol İzleyicisi (Hotkey Monitor)

## Genel Bakış
`HotkeyMonitor` (`src/victus-max-daemon/src/hotkey_monitor.rs`), Linux masaüstü ortamından (X11 / Wayland pencere yöneticilerinden) bağımsız olarak, doğrudan Linux çekirdeğinin `evdev` girdi alt sistemi üzerinden çalışan küresel kısayol yakalayıcıdır.

## Mimari ve Algılama Yöntemi

1. **Aygıt Taraması (`/dev/input/`):** Başlangıçta tüm girdi düğümlerini tarar, klavye yeteneğine sahip (`KEY_A`, `KEY_F2`, `KEY_PROG1`, `KEY_CALC` destekleyen) ve fare olmayan aygıtları tespit eder.
2. **Akış Birleştirme (`futures::stream::select_all`):** Birden fazla dahili ve harici klavye varsa tüm girdi akışları tek bir asenkron kuyrukta birleştirilir.
3. **Kombinasyon Takibi (Shift + F2):**
   - `KEY_LEFTSHIFT` veya `KEY_RIGHTSHIFT` basılı tutulduğunda durum bayrağı aktif edilir.
   - Shift basılıyken `KEY_F2` tuşuna basıldığı anda oyun içi HUD arayüzünü açmak üzere [[platform-service]] üzerinden `toggle_overlay()` fonksiyonu tetiklenir veya D-Bus sinyali yayılır.
4. **Özel OMEN Tuşu:** Bazı modellerde bulunan fiziksel OMEN tuşunu (`KEY_PROG1`) yakalayarak ana kontrol merkezini ([[gui-application]]) öne getirir.

## Avantajları
Masaüstü ortamlarının (GNOME/KDE) küresel kısayol çakışmalarından veya Wayland'in güvenlik kısıtlamalarından etkilenmez; tam ekran oyunların içindeyken dahi mikro-saniye gecikmeyle çalışır.

## İlgili Bağlantılar
- Oyun İçi HUD: [[quick-hud-overlay]]
- Platform Servisi: [[platform-service]]
- Girdi İzinleri: [[udev-device-rules]]
