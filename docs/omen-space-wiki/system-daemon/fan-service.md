# Fan Kontrol Servisi ve Termal Koruma Motoru

## Genel Bakış
`FanService` (`src/omen-space-daemon/src/fan/mod.rs`), sistem fanlarının hızlarını, BIOS müdahalelerini, özel eğri interpolasyonlarını ve acil durum termal korumasını yöneten en kritik arka plan motorudur.

Servis, `org.hp.omen.Fan` D-Bus arayüzü ([[fan-dbus-interface]]) üzerinden istemcilerden gelen istekleri alır ve donanım katmanında [[embedded-controller-ec]] veya [[hp-wmi-driver]] kanallarına iletir.

## Çalışma Modları

1. **Auto (Otomatik):** BIOS'un veya dahili histerezis algoritmasının fan hızını yönettiği varsayılan mod.
2. **Max (Maksimum):** Fanların %100 PWM görev döngüsü ve tam devirde (RPM) çalıştığı soğutma modu.
3. **Custom (Özel Eğri):** Kullanıcı tarafından tanımlanan sıcaklık-devir noktalarına göre dinamik spline interpolasyonu uygulayan mod.
4. **Manual Percentage:** Kullanıcının terminalden veya arayüzden tekil bir yüzde (örneğin `%60`) belirlediği durum.

## Termal Koruma Mekanizması (Emergency Thermal Protection)

Donanımın aşırı ısınmasını ve termal hasarı önlemek amacıyla serviste otonom bir güvenlik devresi bulunur:
- **Tetiklenme Sıcaklığı (`THERMAL_PROTECTION_TRIGGER_TEMP`):** CPU veya GPU sıcaklığı **95.0°C** eşiğine ulaştığında veya aştığında termal koruma devreye girer.
- **Acil Durum Eylemi:** Mevcut fan modu ne olursa olsun, fanlar derhal **%100 Max** hızına çekilir.
- **Kullanıcı Bildirimi:** Sistem `DesktopNotifier` ile masaüstü uyarısı gönderir ve D-Bus üzerinden `thermal_protection_alert(true)` sinyali yayar.
- **İyileşme Eşiği (`THERMAL_PROTECTION_RECOVER_TEMP`):** Sıcaklık **82.0°C** altına düşene kadar fanlar maksimumda tutulur. Eşiğin altına inildiğinde sistem önceki moda geri döner.

## Özel Fan Eğrisi ve İnterpolasyon

Kullanıcı arayüzünde ([[fan-curve-editor-ui]]) oluşturulan eğriler `Vec<CurvePoint>` olarak JSON formatında iletilir:
- Her nokta `CurvePoint(Temp, Pct)` ikilisidir (örneğin `[45.0, 20.0]`, `[75.0, 65.0]`, `[85.0, 100.0]`).
- Servis, o anki sıcaklığı bu noktalar arasında lineer veya kübik spline ile hesaplayarak hedef PWM değerini belirler.
- **Tepe Tutma (`AUTO_PEAK_HOLD_SECS = 15`):** Fanların anlık sıcaklık sıçramalarında sürekli hızlanıp yavaşlamasını (fan hunting) önlemek için tepe hız 15 saniye boyunca korunur.

## Keep-Alive ve BIOS Geçersiz Kılma
HP anakartları belirli bir süre müdahale edilmediğinde fan kontrolünü tekrar BIOS'a devretme eğilimindedir. `FanService`, arka plan döngüsünde periyodik olarak donanım kayıtlarını tazeleyerek kullanıcının seçtiği modun aktif kalmasını güvenceye alır.

## İlgili Bağlantılar
- Donanım Katmanı: [[embedded-controller-ec]], [[hp-wmi-driver]]
- D-Bus Sözleşmesi: [[fan-dbus-interface]]
- Görsel Editör: [[fan-curve-editor-ui]]
- Mimari Karar: [[adr-002-wmi-vs-direct-ec-arbitration]]
