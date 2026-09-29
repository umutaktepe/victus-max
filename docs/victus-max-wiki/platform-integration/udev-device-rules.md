# Udev Donanım İzinleri ve Aygıt Kuralları

## Genel Bakış
Linux çekirdeği varsayılan olarak donanım MSR yazmaçlarını (`/dev/cpu/*/msr`) ve doğrudan girdi akışlarını (`/dev/input/event*`) yalnızca root erişimine açık tutar veya belirli gruplara kısıtlar.

`data/99-omen-space.rules` dosyası, Victus Max servislerinin ve `omen-hw` grubunun ihtiyaç duyduğu donanım aygıt izinlerini kalıcı hale getirir.

## Udev Kural Dosyası (`data/99-omen-space.rules`)

```udev
# Intel/AMD MSR aygıtları okuma/yazma izinleri
KERNEL=="msr*", GROUP="omen-hw", MODE="0660"

# Gömülü girdi aygıtları (Kısayol ve OMEN tuşu dinleme)
KERNEL=="event*", SUBSYSTEM=="input", GROUP="omen-hw", MODE="0660"

# hp-omen-extra ve platform sysfs izinleri
SUBSYSTEM=="platform", DRIVERS=="hp-omen-extra", GROUP="omen-hw", MODE="0664"
```

## Güvenlik ve Donanım Yalıtımı
- MSR aygıtlarına yazma izni Linux çekirdeğinde varsayılan olarak güvenlik amacıyla kısıtlanmıştır. Bu kural sayesinde `victus-max-daemon` güvenli biçimde [[cpu-msr-undervolt]] ve [[amd-ryzen-smu]] modüllerini işletebilir.
- [[hotkey-monitor]] girdi olaylarını dinlerken aygıt kilitlenmesi yaşamadan çalışabilir.

## İlgili Bağlantılar
- Intel MSR Donanımı: [[cpu-msr-undervolt]]
- AMD SMU Donanımı: [[amd-ryzen-smu]]
- Kısayol İzleyici: [[hotkey-monitor]]
