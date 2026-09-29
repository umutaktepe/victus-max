# Paketleme, DKMS ve Dağıtım Altyapısı

## Genel Bakış
Victus Max; Arch Linux (AUR), NixOS (Flakes), Ubuntu/Debian, Fedora ve openSUSE dahil olmak üzere başlıca Linux dağıtımlarında sorunsuz kurulup güncellenebilecek şekilde tasarlanmıştır.

## Derleme ve İkili Optimizasyon (`Cargo.toml`)
Kök dizindeki `Cargo.toml` yapılandırması, minimum ikili dosya boyutu ve maksimum çalışma hızı için optimize edilmiştir:

```toml
[profile.release]
opt-level = "z"     # Boyut optimizasyonu
lto = true          # Bağlama Zamanı Optimizasyonu (Link-Time Optimization)
codegen-units = 1   # Maksimum satır içi optimizasyon
panic = "abort"     # Yığın çözme (unwinding) overhead'ini kaldır
strip = true        # Sembol tablolarını ikili dosyadan ayıkla
```
Bu ayarlar sayesinde daemon ikili boyutu ~3MB'a ve bellek tüketimi <5MB seviyesine iner.

## DKMS Modül Yapılandırması (`driver/dkms.conf`)

[[hp-omen-extra-dkms]] kernel sürücüsünün otomatik derlenmesi için DKMS manifestosu:

```ini
PACKAGE_NAME="hp-omen-extra"
PACKAGE_VERSION="2.1.2"
BUILT_MODULE_NAME[0]="hp-omen-extra"
DEST_MODULE_LOCATION[0]="/kernel/drivers/platform/x86"
AUTOINSTALL="yes"
```
Her yeni çekirdek kurulduğunda veya güncellendiğinde DKMS modülü otomatik olarak hedef çekirdek için inşa eder.

## Dağıtım Formatları

1. **Arch Linux (`PKGBUILD`):** AUR üzerinden tek komutla (`makepkg -si`) tüm workspace paketlerini derler ve systemd servislerini devreye alır.
2. **NixOS (`flake.nix`):** Bildirime dayalı (declarative) Nix flake desteği ile sıfır dış bağımlılık sorunu.
3. **Otomatik Web Yükleyici (`install.sh`):** Tek satırlık web komutu (`curl -sSL ... | sudo bash`) dağıtım paket yöneticisini (`pacman`, `apt`, `dnf`, `zypper`) tespit ederek gerekli bağımlılıkları yükler ve `setup.sh` betiğini çalıştırır.

## İlgili Bağlantılar
- Donanım Sürücüsü: [[hp-omen-extra-dkms]]
- Mimari Karar: [[adr-003-companion-dkms-driver-non-clashing]]
- Çekirdek Servis: [[daemon-overview]]
