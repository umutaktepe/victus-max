# Polkit ve D-Bus Güvenlik Politikası

## Genel Bakış
Victus Max'ın ayrıcalık izolasyonu modelinde ([[adr-001-rust-daemon-client-split]]), root yetkili daemon ile unprivileged kullanıcı istemcileri arasındaki erişim sınırları D-Bus güvenlik politikası (`data/org.hp.omen.conf`) ile belirlenir.

## D-Bus Güvenlik Politikası (`data/org.hp.omen.conf`)

Dosya `/usr/share/dbus-1/system.d/org.hp.omen.conf` (veya `/etc/dbus-1/system.d/`) altına yerleştirilir:

```xml
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <!-- Sadece root kullanıcısı servisi sahiplenebilir -->
  <policy user="root">
    <allow own="org.hp.omen"/>
    <allow send_destination="org.hp.omen"/>
    <allow receive_sender="org.hp.omen"/>
  </policy>

  <!-- omen-hw ve wheel/sudo grubundaki kullanıcılar komut gönderebilir -->
  <policy group="omen-hw">
    <allow send_destination="org.hp.omen"/>
    <allow receive_sender="org.hp.omen"/>
  </policy>

  <policy group="wheel">
    <allow send_destination="org.hp.omen"/>
    <allow receive_sender="org.hp.omen"/>
  </policy>

  <policy group="sudo">
    <allow send_destination="org.hp.omen"/>
    <allow receive_sender="org.hp.omen"/>
  </policy>

  <!-- Diğer tüm unprivileged kullanıcılar için varsayılan kısıtlama -->
  <policy context="default">
    <deny send_destination="org.hp.omen"/>
  </policy>
</busconfig>
```

## Güvenlik Sağlaması
1. **İsim Gaspını Önleme (No Name Spoofing):** Yalnızca `root` kullanıcısı `org.hp.omen` veri yolu adını alabilir. Yetkisiz bir yerel süreç kendisini Victus Max daemon gibi tanıtamaz.
2. **Kötü Niyetli Yazılımları Engelleme:** Sistemde çalışan üçüncü parti korumasız web tarayıcıları veya izole edilmemiş scriptler donanım fanlarını kapatamaz veya voltaj değerlerini bozamaz.
3. **Kullanıcı Kolaylığı:** Kullanıcı `omen-hw` veya `wheel` grubundaysa her fan/profil değişiminde şifre (sudo prompt) sormadan akıcı bir deneyim yaşar.

## Polkit Güncelleme Politikası (`org.hp.victusmax.update`)

Sistem bileşenlerinin ve çekirdek sürücülerinin grafik arayüz üzerinden root yetkisiyle güncellenmesi için PolicyKit eylemi kullanılır ([[updater-service]]):

- **Politika Dosyası:** `/usr/share/polkit-1/actions/org.hp.victusmax.update.policy`
- **Hedef Betik:** `/usr/libexec/victus-max/victus-max-updater`
- **Yetki Kuralı:** `auth_admin_keep` (yönetici oturum açtığında parola belirli bir süre önbelleğe alınır).
- **GUI İzni:** `org.freedesktop.policykit.exec.allow_gui = true`

## İlgili Bağlantılar
- Mimari Karar: [[adr-001-rust-daemon-client-split]]
- OTA Güncelleme Kararı: [[adr-006-github-update-and-release-architecture]]
- Güncelleme Merkezi: [[updater-service]]
- D-Bus Protokolü: [[dbus-ipc-protocol]]
- Çekirdek Servis: [[daemon-overview]]
