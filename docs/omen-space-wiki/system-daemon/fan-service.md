# Fan Kontrol Servisi ve Termal Koruma Motoru

## Genel Bakış
`FanService` (`src/omen-space-daemon/src/fan/mod.rs`), sistem fanlarının hızlarını, BIOS müdahalelerini, özel eğri interpolasyonlarını ve acil durum termal korumasını yöneten en kritik arka plan motorudur.

Servis, `org.hp.omen.Fan` D-Bus arayüzü ([[fan-dbus-interface]]) üzerinden istemcilerden gelen istekleri alır ve donanım katmanında [[embedded-controller-ec]] veya [[hp-wmi-driver]] kanallarına iletir.

## Çalışma Modları

1. **Auto (Otomatik):** BIOS'un veya dahili histerezis algoritmasının fan hızını yönettiği varsayılan mod.
2. **Better Auto (Proaktif Otomatik):** CPU yükü (`/proc/stat` delta) ve sıcaklık çift matrisini izleyerek sıcaklık yükselmeden fan devrini proaktif artıran, iniş sınırlayıcılı (single-step ramp-down), ayarlanabilir asgari RPM (varsayılan 2600 RPM) ve akustik tavanlı (Balanced modunda varsayılan Seviye 5 / ~4100 RPM) akıllı fan motoru.
3. **Max (Maksimum):** Fanların %100 PWM görev döngüsü ve tam devirde (RPM) çalıştığı soğutma modu.
4. **Custom (Özel Eğri):** Kullanıcı tarafından tanımlanan sıcaklık-devir noktalarına göre dinamik spline interpolasyonu uygulayan mod.
5. **Manual Percentage:** Kullanıcının terminalden veya arayüzden tekil bir yüzde (örneğin `%60`) belirlediği durum.

## Proaktif Better Auto Algoritması ve Akustik Tavan
`BetterAutoEngine` (`src/victus-max-daemon/src/fan/better_auto.rs`), `victus-control` projesinin proaktif iş yükü algısını Rust mimarisine kazandırır:
- **Çift Matrisli Seviye Tespiti:** Sistem sıcaklığı (CPU/GPU) ve işlemci yükü (CPU delta) bağımsız 8 kademeli baremlerde değerlendirilir ve büyük olan seviye seçilir. Yük %70 üzerine fırladığında sıcaklık henüz 50°C olsa bile fanlar anında Seviye 6'ya yükselir.
- **Asgari Devir Tabanı (Minimum RPM):** Balanced profilinde fanlar hem CPU hem GPU için varsayılan **2600 RPM** tabanında tutulur. Kullanıcı bunu GUI ve CLI üzerinden 2000–3500 RPM arasında değiştirebilir.
- **Akustik Tavan (Acoustic Ceiling):** Balanced modunda gürültüyü sınırlamak için fan seviyesi kullanıcının seçtiği akustik tavana (varsayılan Seviye 5) sınırlandırılır. `performance` profilinde tavan Seviye 8'e (%100) çıkarılır.
- **88°C Termal Baypas (Thermal Bypass):** Donanım sıcaklığı 88.0°C ve üzerine ulaştığında akustik tavan ve bekleme süreleri tamamen göz ardı edilir; fanlar anında Seviye 8'e çekilir.
- **EC Güvenliği ve Koruması (10s Stagger Gap):** HP Victus EC veri yolunun kilitlenmesini engellemek için Fan 1 ve Fan 2 yazımları arasında asenkron 10 saniyelik gecikme uygulanır ([[adr-005-better-auto-proactive-fan-and-victus-max]]).
- **90s Watchdog Tazelemesi:** BIOS'un fan kontrolünü zorla devralmasını önlemek için her 90 saniyede bir manuel fan hedefleri EC'ye yeniden yazılır.

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
- Mimari Kararlar: [[adr-002-wmi-vs-direct-ec-arbitration]], [[adr-005-better-auto-proactive-fan-and-victus-max]]
