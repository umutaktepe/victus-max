# Gömülü Denetleyici (Embedded Controller - EC) Doğrudan IO Arayüzü

## Genel Bakış
Gömülü Denetleyici (EC), modern dizüstü bilgisayarlarda anakart üzerinde bağımsız çalışan, güç dağıtımı, şarj yönetimi, klavye matrisi ve doğrudan fan hız kontrolünden sorumlu mikroişlemcidir.

OMEN Space içerisinde `LinuxEcController` (`src/victus-max-daemon/src/ec.rs`), Linux çekirdeğinin `ec_sys` sürücüsü aracılığıyla sağladığı debugfs IO dosyasını (`/sys/kernel/debug/ec/ec0/io`) kullanarak donanım kayıtlarına (registers) doğrudan erişim sağlar.

## Bellek Kayıt Haritası (Register Map)

`ec.rs` içerisinde tanımlanan kritik HP Omen/Victus EC adresleri şunlardır:

| Kayıt Sabiti | Adres (Hex) | İşlev |
| :--- | :--- | :--- |
| `REG_FAN1_SPEED_PCT` | `0x2E` | 1. Fan (CPU) mevcut görev döngüsü (%) |
| `REG_FAN2_SPEED_PCT` | `0x2F` | 2. Fan (GPU) mevcut görev döngüsü (%) |
| `REG_FAN1_SPEED_SET` | `0x34` | 1. Fan hedef hız yazma kaydı |
| `REG_FAN2_SPEED_SET` | `0x35` | 2. Fan hedef hız yazma kaydı |
| `REG_CPU_TEMP` | `0x57` | EC tarafından algılanan CPU sıcaklığı (°C) |
| `REG_BIOS_CONTROL` | `0x62` | Fan BIOS otomatik denetim bayrağı |
| `REG_PERF_MODE` | `0x95` | Donanımsal performans modu kayıt bayrağı |
| `REG_GPU_TEMP` | `0xB7` | EC tarafından algılanan GPU sıcaklığı (°C) |
| `REG_FAN_BOOST` | `0xEC` | Maksimum fan hız artırma (Boost) tetikleyicisi |
| `REG_FAN_STATE` | `0xF4` | Fan çalışma durumu göstergesi |

## Donanım Güvenlik Duvarı ve Tahkim (Safety Firewall)

Doğrudan EC erişimi muazzam hız sağlamakla birlikte, uyumsuz BIOS/EC firmware mimarilerinde ölümcül kilitlenmelere yol açabilir. Bu nedenle katı bir güvenlik protokolü uygulanmaktadır:

### 1. Güvensiz Model Kara Listesi (`UNSAFE_MODELS` & `UNSAFE_BOARDS`)
Aşağıdaki modellerde doğrudan EC yazımları donanımsal Caps Lock yanıp sönmesiyle sonuçlanan firmware paniklerine neden olduğu için kesinlikle engellenir:
- **Kasa Modelleri:** `16t-ah0`, `16-ah0`, `16-ap0`, `17t-ah0`, `17-ah0`, `transcend 14`
- **Anakart Kimlikleri:** `8c58`, `8d24`
- **Davranış:** Bu sistemlerde `is_unsafe_model = true` bayrağı set edilir ve tüm fan kontrolü [[hp-wmi-driver]] üzerine delege edilir. (Bkz: [[adr-002-wmi-vs-direct-ec-arbitration]]).

### 2. hwmon Fallback Beyaz Listesi
Bazı Victus sistemlerinde standart çekirdek `hwmon` aygıtlarında `fan_input` RPM düğümleri bulunmaz:
- **Desteklenen Anakartlar:** `8A42`, `8A43`, `8E35`
- **Çözüm:** `needs_ec_fallback_for_board()` fonksiyonu bu anakartları tespit eder ve `0x2E` / `0x2F` görev döngüsü değerlerini telemetri için RPM yaklaşık değerine dönüştürür.

## Mount ve Debugfs Yönetimi
Eğer `/sys/kernel/debug/ec/ec0/io` dosyası mevcut değilse, denetleyici sistem başlatma esnasında `mount -t debugfs none /sys/kernel/debug` komutunu güvenli biçimde yürüterek debugfs dosya sistemini bağlamayı dener.

## İlgili Bağlantılar
- Mimari Karar: [[adr-002-wmi-vs-direct-ec-arbitration]]
- Tüketici Servis: [[fan-service]]
- WMI Alternatifi: [[hp-wmi-driver]]
- Donanım Matrisi: [[board-capabilities-matrix]]
