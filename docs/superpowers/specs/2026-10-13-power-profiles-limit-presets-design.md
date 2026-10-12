# Victus Max — Güç Modu Profilleri ve Donanımsal Limit Koruma Tasarım Dokümanı (Design Spec)

**Tarih:** 2026-10-13  
**Durum:** Onaylandı / Tasarım Aşamasında  
**İlgili Kararlar:** [[adr-004-native-msr-and-smu-mailbox-tuning]], [[adr-007-adaptive-ryzen-power-limits]], [[adr-008-power-profile-limit-presets]]

---

## 1. Amaç ve Problem Tanımı

Kullanıcıların kendi Victus/OMEN sistemlerinde `ryzenadj` ve `power-profiles-daemon` kullanarak oluşturdukları kabuk scriptleri (30 sn watchdog timer, udev AC tak/çıkar tetikleyicisi, uykudan uyanma servisi, GNOME/KDE profil dinleyicisi), HP BIOS/EC'nin işlemci güç limitlerini arka planda sessizce fabrika ayarlarına çekmesini engellemekte ve belirli güç modlarında (Power Save, Balanced, Performance) istenen termal ve güç tavanlarını garanti altına almaktadır.

Bu yeteneğin harici bash betiklerine ve dağınık systemd servislerine ihtiyaç kalmaksızın, doğrudan `victus-max-daemon`, `victus-max-gui` ve `victus-max-cli` bileşenlerine hem **AMD Ryzen** hem de **Intel** mimarilerini tam destekleyecek şekilde yerel olarak entegre edilmesi hedeflenmektedir.

---

## 2. Mimari Kararlar ve İlkeler

1. **Tam Daemon Entegrasyonu (No External Bash Scripts):**
   - Tüm durum yönetimi, emniyet kilitleri ve zamanlayıcılar `victus-max-daemon` bünyesinde Rust ile yürütülür.
   - Yapılandırma `/etc/victus-max/power.json` altında `limit_presets` anahtarıyla kalıcı olarak tutulur.
2. **30 Saniyelik Emniyet Kilidi (Silent Watchdog Loop):**
   - HP EC/BIOS'un arka planda limitleri sıfırlama ihtimaline karşı `PowerService` her 30 saniyede bir aktif modun limitlerini sessizce doğrular ve uygular.
3. **Donanım Olayları ve Uyku Entegrasyonu (AC & Suspend/Resume):**
   - AC şarj aletinin takılması/çıkarılması durumunda gecikmesiz limit güncellemesi yapılır (`power_automation.rs`).
   - `org.freedesktop.login1` (systemd-logind) `PrepareForSleep` sinyali dinlenerek sistem uykudan (suspend/hibernate) uyandığı an limitler anında tazelenir.
4. **Çift Yönlü PPD / Performans Modu Eşitlemesi:**
   - GNOME/KDE paneli, `powerprofilesctl`, Victus Max Performans sekmesi veya pil durumu üzerinden mod değiştiğinde, Advanced Power sekmesindeki güç limiti modu da seçilen moda otomatik geçer ve ilgili limitleri uygular.
5. **Slider Override Hiyerarşisi:**
   - 3 moddan biri seçiliyken, kullanıcı üstteki slider'lardan herhangi birini değiştirip "Apply & Save" dediğinde aktif mod seçimi kalkar ve sistem "Manuel Override" durumuna geçer.
   - Alttaki 3 moddan birine tıklandığında ise mod slider'ları ezer; slider değerleri o modun kayıtlı değerlerine çekilir ve mod aktif edilir.
6. **İşlemci Tabanlı Hibrit Varsayılanlar (Factory Defaults):**
   - İlk kurulumda donanımın anlık değerleri sorgulanır.
   - `cpu_tier.rs` sınıflandırmasına göre (Ryzen HS/HX/U, Intel H/HX/Standart) kalibre edilmiş varsayılanlar atanır:
     - **Ryzen Standart HS (örn. 7840HS):**
       - Power Save: 20W STAPM / 20W Slow / 25W Fast / 70°C
       - Balanced: 35W STAPM / 35W Slow / 35W Fast / 82°C
       - Performance: 45W STAPM / 45W Slow / 45W Fast / 88°C
     - **Intel Standart H (örn. 13500H / 13700H):**
       - Power Save: 25W PL1 / 35W PL2 / 75°C (TCC 25)
       - Balanced: 45W PL1 / 65W PL2 / 85°C (TCC 15)
       - Performance: 55W PL1 / 95W PL2 / 97°C (TCC 3)
   - Her mod düzenleme penceresinde fabrika ayarlarına döndüren "Varsayılana Sıfırla" düğmesi yer alır.

---

## 3. D-Bus Arayüz Sözleşmesi (`org.hp.omen.Power`)

```rust
// Yeni Metotlar
GetPowerPresets() -> String (JSON)
SetPowerPresetMode(mode: String) -> String ("OK" / "FAIL")
SavePowerPreset(mode: String, stapm_w: u32, slow_ppt_w: u32, fast_ppt_w: u32, tctl_temp: u32) -> String ("OK" / "FAIL")
ResetPowerPresets() -> String ("OK" / "FAIL")
```

---

## 4. Kullanıcı Arayüzü (GTK4 / Libadwaita)

- Advanced Power sekmesinin en altına "Güç Modu Profilleri" preferences grubu eklenir.
- 3 adet kart/buton: Power Save, Balanced, Performance.
- Her kart üzerinde:
  - Durum ikonu ve mod başlığı
  - Aktif watt/sıcaklık özeti (örn: "35W · 82°C")
  - Düzenleme (Kalem / Edit) butonu.
- Düzenleme butonuna tıklandığında Libadwaita Dialog açılır; Ryzen ise STAPM/Slow/Fast/Tctl, Intel ise PL1/PL2/TCC slider'ları gösterilir.
