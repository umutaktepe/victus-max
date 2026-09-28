# Etkileşimli Fan Eğrisi Düzenleyicisi (Fan Curve Editor UI)

## Genel Bakış
`fan_curve_editor.rs` (`src/victus-max-gui/src/`), GTK4 `DrawingArea` üzerinde çalışan ve kullanıcının fare hareketleriyle sıcaklığa bağlı fan hız eğrilerini (spline) çizmesini sağlayan interaktif bir grafik bileşendir.

## Arayüz Yetenekleri ve Çizim Motoru

1. **Koordinat Düzlemi:**
   - **X Ekseni (Sıcaklık):** 30°C ile 100°C arası çalışma sıcaklığı.
   - **Y Ekseni (Fan Hızı):** %0 ile %100 arası PWM devir yüzdesi.
2. **Nokta Yönetimi (Control Points):** Kullanıcı eğri üzerine çift tıklayarak yeni sıcaklık eşik noktaları ekleyebilir, var olan noktaları sürükleyerek fan tepkisini hassasça ayarlayabilir.
3. **Önceden Tanımlı Hazır Profiller (`fan_presets.rs`):**
   - **Sessiz (Silent):** Düşük sıcaklıklarda fanları tamamen durduran veya fısıltı seviyesinde tutan eğri.
   - **Dengeli (Balanced):** Günlük kullanım ve ofis işleri için optimize edilmiş eğri.
   - **Agresif Oyun (Aggressive Gaming):** 70°C sonrasında hızla %100 devire çıkan yüksek performans eğrisi.

## Veri İletimi
Kullanıcı "Uygula" düğmesine bastığında, arayüzdeki kontrol noktaları JSON formatına dönüştürülür ve [[fan-dbus-interface]] üzerinden [[fan-service]] motoruna iletilir.

## İlgili Bağlantılar
- Grafik Uygulaması: [[gui-application]]
- Fan Servisi: [[fan-service]]
- D-Bus Formatı: [[fan-dbus-interface]]
