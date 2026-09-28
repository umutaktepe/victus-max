# Sistem Tepsisi Uygulaması (System Tray)

## Genel Bakış
`omen-tray` (`src/victus-max-tray/`), masaüstü panelinde (GNOME Tray / AppIndicator, KDE StatusNotifierItem) sessizce çalışan ve kullanıcının ana grafik arayüzünü açmadan temel donanım profillerini değiştirmesini sağlayan hafif bir araçtır.

## Özellikler ve Menü Yapısı

- **Sol Tıklama:** Ana kontrol merkezi olan [[gui-application]] penceresini başlatır veya ön plana getirir.
- **Sağ Tıklama (Hızlı Menü):**
  - **Güç Profilleri:** `Performance`, `Balanced`, `Quiet`.
  - **Fan Modları:** `Auto`, `Max Fan Boost`.
  - **Çıkış:** Tray uygulamasını kapatır.
- **Kaynak Tüketimi:** Tamamen Rust ile yazılmış olup ~2MB gibi önemsiz bir bellek ayak izine sahiptir.

## İlgili Bağlantılar
- Güç Servisi: [[power-service]]
- Fan Servisi: [[fan-service]]
- Ana Grafik Uygulaması: [[gui-application]]
