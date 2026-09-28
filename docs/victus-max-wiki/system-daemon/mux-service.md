# GPU MUX Switch ve Ekran Yönlendirme Servisi

## Genel Bakış
`MuxService` (`src/victus-max-daemon/src/mux.rs`), HP Omen dizüstü bilgisayarlarda yer alan donanımsal multiplexer (MUX) yongasını yöneten servistir.

MUX Switch, dahili laptop ekran panelinin doğrudan harici NVIDIA GPU'ya (Ayrık / Discrete mod) mı yoksa işlemciye entegre iGPU'ya (Hibrit / Optimus mod) mı bağlanacağını belirler. Servis, D-Bus üzerinde `org.hp.omen.Mux` arayüzünü ([[mux-dbus-interface]]) sunar.

## Çalışma Mantığı ve Donanım Yolu

### 1. WMI Aygıt Arabirimi
MUX anahtarlama işlemi, BIOS seviyesinde bir bayrak değiştirilmesiyle yürütülür:
- **Sysfs Yolu:** `/sys/devices/platform/hp-wmi/gpu_mux_mode`
- **Yazılan Değerler:** `discrete` (Ayrık dGPU) veya `hybrid` (Hibrit/Optimus).
- **Entegrasyon:** [[hp-wmi-driver]] aracılığıyla WMI çağrısı tetiklenir.

### 2. Yeniden Başlatma Gereksinimi (Reboot Required)
Donanımsal MUX anahtarlaması ekran panelinin fiziksel veri yolu bağlantısını değiştirdiğinden, işletim sistemi yeniden başlatılmadan tam olarak uygulanamaz:
- `SetGpuMode("discrete")` çağrısı başarılı olduğunda servis `"OK_REBOOT_REQUIRED"` yanıtı döner.
- Kullanıcı arayüzünde ([[gui-application]]) anında bir yeniden başlatma onay penceresi tetiklenir.

### 3. Ekran ve GPU Telemetrisi
Servis, sistemdeki aktif ekran çıkışlarını (`/sys/class/drm/`) ve bağlı panelleri tarayarak hangi GPU'nun hangi ekranı sürdüğünü raporlar.

### 4. Yapılandırma Kalıcılığı
Kullanıcının tercih ettiği MUX modu `/etc/omen-space/mux.json` dosyasında kaydedilir.

## İlgili Bağlantılar
- Donanım Katmanı: [[hp-wmi-driver]]
- D-Bus Sözleşmesi: [[mux-dbus-interface]]
- Grafik İstemcisi: [[gui-application]]
