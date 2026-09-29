# Oyun ve Uygulama Otomasyon Servisi (Game Automation)

## Genel Bakış
`GameAutomationService` (`src/victus-max-daemon/src/game_automation.rs`), sistemde belirli oyunlar veya ağır iş yükü uygulamaları başlatıldığında güç profilini ve fan modunu otomatik olarak ayarlayan arka plan servisidir.

Servis, D-Bus üzerinde `org.hp.omen.AppProfiles` arayüzünü sunar ve `/etc/victus-max/app_profiles.json` dosyasında saklanan profil kurallarını (eski `/etc/omenspace/app_profiles.json` fallback desteğiyle, bkz: [[app-profiles-spec]]) uygular.

## Çalışma Mekanizması

1. **Süreç İzleme Döngüsü (`start_monitor`):** Arka planda periyodik olarak çalışan işlemler taranır.
2. **Profil Eşleme:** Kullanıcının tanımladığı profil listesindeki bir ikili dosya adı (örneğin `cs2`, `cyberpunk2077`, `blender`) tespit edildiğinde:
   - [[power-service]] çağrılarak belirlenen güç profili (örneğin `performance`) aktif edilir.
   - [[fan-service]] çağrılarak hedeflenen fan modu (örneğin `max` veya `custom`) tetiklenir.
3. **Masaüstü Bildirimi:** `DesktopNotifier` ile kullanıcının ekranında "Oyun Profili Devrede" bildirimi gösterilir.
4. **Geri Dönüş (Restoration):** Uygulama sonlandırıldığında sistem otomatik olarak önceki varsayılan güç ve fan ayarlarına döner.

## İlgili Bağlantılar
- Veri Şeması: [[app-profiles-spec]]
- Güç Servisi: [[power-service]]
- Fan Servisi: [[fan-service]]
- Arayüz Sekmesi: [[gui-application]]
