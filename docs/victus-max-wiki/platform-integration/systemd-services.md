# Systemd Servisleri ve Arka Plan Yaşam Döngüsü

## Genel Bakış
OMEN Space, Linux sistem başlatma ve servis yönetiminde `systemd` altyapısını kullanır.

Sistem, root yetkileriyle donanımı kontrol eden sistem servisi (`omen-space-daemon.service`) ile masaüstü oturumunda D-Bus aktivasyonunu sağlayan kullanıcı servislerinden (`org.hp.OmenSpace.service`) oluşur.

## Sistem Servis Dosyası (`data/omen-space-daemon.service`)

```ini
[Unit]
Description=OMEN Space Hardware Control Daemon
After=network.target dbus.service
Wants=dbus.service

[Service]
Type=simple
ExecStart=/usr/bin/omen-space-daemon
Restart=always
RestartSec=2
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

### Kritik Özellikler
- **`Restart=always`:** Beklenmeyen bir panik veya geçici donanım hatasında fan kontrolünün ve termal korumanın devre dışı kalmaması için 2 saniye içinde servis otomatik yeniden başlatılır.
- **`After=dbus.service`:** `zbus` System Bus kaydı yapabilmek için D-Bus servisinin aktif olması şart koşulmuştur.

## D-Bus Servis Aktivasyonu (`data/org.hp.OmenSpace.service`)
Kullanıcı grafik uygulamayı (`omen-gui`) veya komut satırını çalıştırdığında, eğer D-Bus nesnesi henüz uykudaysa sistemin D-Bus aktivasyonuyla ilgili bileşeni ayağa kaldırmasını sağlar.

## Kullanıcı ve Grup Tanımlaması (`data/sysusers.d/omen-space.conf`)
Paket kurulumunda `sysusers.d` kuralı ile `omen-hw` adında bir sistem grubu oluşturulur. Bu grup, unprivileged kullanıcıların D-Bus üzerinden donanım komutu göndermesini yetkilendirmek için kullanılır.

## İlgili Bağlantılar
- Çekirdek Servis: [[daemon-overview]]
- Güvenlik ve Yetkilendirme: [[polkit-dbus-security]]
