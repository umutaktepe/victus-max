# HP WMI Çekirdek Sürücüsü ve ACPI Arayüzü

## Genel Bakış
HP WMI sürücüsü (`hp-wmi`), Linux çekirdeğinin HP dizüstü bilgisayarlardaki donanım yönetim arabirimini ACPI (Advanced Configuration and Power Interface) WMI GUID'leri üzerinden kontrol eden resmi mekanizmasıdır.

OMEN Space ekosisteminde `hp-wmi`, çekirdeğin sunduğu standart sysfs aygıt düğümlerini (`/sys/devices/platform/hp-wmi/`) okuyup yazarak BIOS seviyesinde termal profilleri, GPU MUX yönlendirmesini ve donanım olay bildirimlerini yönetmek için temel zemin oluşturur.

## Çekirdek Mimarisi ve Arayüz Yolları

### 1. Termal Profil Yönetimi
HP BIOS, fan hızlarını ve işlemci TDP hedeflerini donanımsal olarak gruplayan termal profilleri WMI üzerinden sunar.
- **Sysfs Yolu:** `/sys/devices/platform/hp-wmi/thermal_profile`
- **Desteklenen Modlar:** `Default` (Dengeli), `Performance` (Yüksek Performans), `Cool` (Sessiz/Serin).
- **Entegrasyon:** [[platform-service]] ve [[power-service]], kullanıcının talebine veya AC güç durumuna göre bu sysfs düğümünü asenkron olarak günceller.

### 2. Grafik Modu ve MUX Switch
Donanımsal MUX switch barındıran Omen serisi sistemlerde dGPU (Ayrık GPU) ve iGPU (Dahili GPU) arasındaki ekran paneli yönlendirmesi HP WMI üzerinden yürütülür.
- **Sysfs Yolu:** `/sys/devices/platform/hp-wmi/gpu_mux_mode`
- **Değerler:** `hybrid` (NVIDIA Optimus / Hibrit) ve `discrete` (Yalnızca dGPU).
- **Davranış:** Mod değişimi yapıldığında sistem hemen yeniden başlatma bayrağı döner. Bu mekanizma doğrudan [[mux-service]] tarafından koordine edilir.

### 3. WMI Olay Bildirimleri ve Tuş Olayları
HP WMI sürücüsü, ACPI olaylarını (Örn: OMEN tuşu, Fn kısayolları, güç fişi durumu) yakalayarak input katmanına aktarır. Ancak modern OMEN modellerinde düşük gecikmeli kısayol yakalama amacıyla [[hotkey-monitor]] doğrudan `/dev/input/event*` aygıtlarını evdev ile dinler.

## Mimari İlişkiler ve Güvenlik
- **Tahkim Mekanizması:** Doğrudan donanım yazımı riskli olan anakartlarda WMI birincil güvenli kanal olarak kullanılır. Detaylar için bkz: [[adr-002-wmi-vs-direct-ec-arbitration]].
- **Yoldaş Modül Uyumu:** Klavye RGB kontrolü bu sürücüde bulunmadığından, WMI GUID çakışması yaşamadan yan yana çalışan [[hp-omen-extra-dkms]] geliştirilmiştir. Karar gerekçesi: [[adr-003-companion-dkms-driver-non-clashing]].

## İlgili Bağlantılar
- Mimari Karar: [[adr-002-wmi-vs-direct-ec-arbitration]]
- Yoldaş Sürücü: [[hp-omen-extra-dkms]]
- GPU Servisi: [[mux-service]]
- Platform Servisi: [[platform-service]]
