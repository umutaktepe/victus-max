# Systemd Servisleri ve Arka Plan Yaşam Döngüsü

## Genel Bakış
Victus Max, Linux sistem başlatma ve servis yönetiminde `systemd` altyapısını kullanır.

Sistem, root yetkileriyle donanımı kontrol eden sistem servisi (`victus-max-daemon.service`, geriye dönük `omen-space-daemon.service` alias ile) ile masaüstü oturumunda D-Bus aktivasyonunu sağlayan kullanıcı servislerinden (`org.hp.VictusMax.service` ve `org.hp.OmenSpace.service`) oluşur.

## Sistem Servis Dosyası (`data/victus-max-daemon.service`)

```ini
[Unit]
Description=Victus Max Hardware Control Daemon
After=network.target dbus.service
Wants=dbus.service

[Service]
Type=simple
ExecStart=/usr/libexec/victus-max/victus-max-daemon
Restart=always
RestartSec=2
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
Alias=omen-space-daemon.service
```

### Kritik Özellikler
- **`Restart=always`:** Beklenmeyen bir panik veya geçici donanım hatasında fan kontrolünün ve termal korumanın devre dışı kalmaması için 2 saniye içinde servis otomatik yeniden başlatılır.
- **`After=dbus.service`:** `zbus` System Bus kaydı yapabilmek için D-Bus servisinin aktif olması şart koşulmuştur.

## D-Bus Servis Aktivasyonu (`data/org.hp.VictusMax.service`)
Kullanıcı grafik uygulamayı (`victus-max`) veya komut satırını çalıştırdığında, eğer D-Bus nesnesi henüz uykudaysa sistemin D-Bus aktivasyonuyla ilgili bileşeni ayağa kaldırmasını sağlar.

## Kullanıcı ve Grup Tanımlaması (`data/sysusers.d/omen-space.conf`)
Paket kurulumunda `sysusers.d` kuralı ile `omen-hw` adında bir sistem grubu oluşturulur. Bu grup, unprivileged kullanıcıların D-Bus üzerinden donanım komutu göndermesini yetkilendirmek için kullanılır.

## İlgili Bağlantılar
- Çekirdek Servis: [[daemon-overview]]
- Güvenlik ve Yetkilendirme: [[polkit-dbus-security]]
