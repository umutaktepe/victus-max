# Sistem Tepsisi Uygulaması (System Tray)

## Genel Bakış
`victus-max-tray` (`src/victus-max-tray/`), masaüstü panelinde (GNOME Tray / AppIndicator, KDE StatusNotifierItem) sessizce çalışan ve kullanıcının ana grafik arayüzünü açmadan temel donanım profillerini ve proaktif soğutma modlarını değiştirmesini sağlayan hafif bir Rust mikro-istemcisidir.

## Özellikler ve Menü Yapısı

- **Sol Tıklama:** Ana kontrol merkezi olan [[gui-application]] (`victus-max`) penceresini başlatır veya ön plana getirir.
- **Sağ Tıklama (Hızlı Menü):**
  - **Victus Max'ı Aç:** Ana grafik arayüzü başlatır.
  - **Overlay'i Aç (Shift+F2):** [[quick-hud-overlay]] oyun içi HUD ekranını tetikler.
  - **Güç Profilleri:** `Performans`, `Dengeli`, `Sessiz`.
  - **Fan Modları:** `Better Auto` (Proaktif yük & sıcaklık optimizasyonu), `Otomatik` (BIOS eğrisi), `Maksimum` (100% Turbo Boost), `Donanım (EC)`.
  - **Çıkış:** Tray uygulamasını ve oturumdaki arayüz işlemlerini kapatır.
- **Konfigürasyon Desteği:** `~/.config/victus-max/gui_config.json` yolundan dil ve ayarları okur (eski `~/.config/omenspace/` dizinine tam geriye dönük uyumluluk sağlar).
- **Kaynak Tüketimi:** Tamamen Rust ile yazılmış olup ~2MB gibi önemsiz bir bellek ayak izine sahiptir.

## İlgili Bağlantılar
- Güç Servisi: [[power-service]]
- Fan Servisi: [[fan-service]]
- Ana Grafik Uygulaması: [[gui-application]]
- Hızlı Oyun İçi HUD: [[quick-hud-overlay]]
