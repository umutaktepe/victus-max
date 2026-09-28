# AMD Ryzen Güç ve Voltaj Optimizasyon Servisi

## Genel Bakış
`RyzenService` (`src/omen-space-daemon/src/ryzen.rs`), AMD Ryzen işlemcili HP Omen ve Victus modellerinde gelişmiş güç ve voltaj optimizasyonunu yürüten servistir.

Servis, D-Bus üzerinde `org.hp.omen.Ryzen` arayüzünü ([[ryzen-dbus-interface]]) sunar ve donanım katmanında [[amd-ryzen-smu]] modülü üzerinden doğrudan SMU posta kutusuyla haberleşir.

## Yönetilen Fonksiyonlar

1. **STAPM Güç Limiti:** Laptop yüzey sıcaklığını kontrol altında tutan uzun vadeli termal güç bütçesi.
2. **Fast & Slow PPT Limitleri:** Oyun veya derleme gibi ağır iş yüklerinde işlemcinin anlık patlama ve sürekli güç tavanlarını belirler.
3. **Tctl Termal Eşiği:** İşlemcinin termal kısmaya girmeden önce ulaşabileceği tepe sıcaklık sınırı.
4. **Curve Optimizer (Tüm Çekirdekler):** İşlemci voltaj-frekans eğrisini negatif yönde kaydırarak daha düşük sıcaklıkta daha yüksek saat hızlarına (boost clocks) ulaşılmasını sağlar.

## Yapılandırma Kalıcılığı
Uygulanan tüm Ryzen optimizasyonları `/etc/omen-space/ryzen.json` dosyasında kaydedilir ve daemon başlatıldığında SMU posta kutusuna taze olarak enjekte edilir.

## Süreç Çakışmalarına Karşı Koruma
Arka planda üçüncü parti güç ayarlayıcıların (`ryzenadj`) çalışması SMU register yazımlarında çakışma yaratabileceğinden servis [[conflict-detector]] ile entegre çalışır.

## İlgili Bağlantılar
- Donanım Katmanı: [[amd-ryzen-smu]]
- D-Bus Sözleşmesi: [[ryzen-dbus-interface]]
- Çakışma Denetimi: [[conflict-detector]]
- Mimari Karar: [[adr-004-native-msr-and-smu-mailbox-tuning]]
