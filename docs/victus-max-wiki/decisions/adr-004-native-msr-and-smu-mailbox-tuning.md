# ADR-004: Yerel (Native) MSR ve SMU Posta Kutusu İnce Ayar Mimarisi

## Durum
Kabul Edildi

## Tarih
2024-09-18 (Revizyon: 2026-09-29)

## Bağlam
HP Omen ve Victus dizüstü bilgisayarlarında hem Intel hem de AMD Ryzen işlemcili varyantlar yaygın olarak bulunmaktadır. İnce ve hafif kasalarda termal kısıtlamaları (thermal throttling) aşmak ve performansı optimize etmek için:
- **Intel İşlemcilerde:** Çekirdek (Core), Grafik (GPU), Önbellek (Cache), Sistem Ajanı (Uncore) voltaj düzlemlerine negatif voltaj ofseti (undervolt) uygulamak ve TCC (Thermal Control Circuit) sıcaklık ofsetini ayarlamak gereklidir.
- **AMD Ryzen İşlemcilerde:** STAPM (Skin Temperature Aware Power Management), Fast PPT, Slow PPT limitlerini ayarlamak ve Curve Optimizer (tüm çekirdeklerde CO negatif ofseti) uygulamak gereklidir.

Linux ekosisteminde bu işlemler geleneksel olarak harici araçlarla (`intel-undervolt`, `throttled`, `ryzenadj` CLI) yürütülmekteydi. Ancak bu harici araç yaklaşımı şu sorunları yaratmaktaydı:
1. **Güvenlik ve Dayanıklılık:** Root yetkili bir daemon'ın sistem komut satırına (shell subprocess) çıkması güvenlik açıkları ve süreç yönetimi riskleri doğurur.
2. **Paketleme ve Bağımlılık Yükü:** Kullanıcının sisteminde ek C/C++ araçlarını derlemesi veya kurması kurulum adımlarını karmaşıklaştırır.
3. **Süreç Çakışmaları:** Arka planda çalışan harici araçlar (`ryzenadj` zamanlayıcıları veya `throttled`), daemon'ın ayarlamaya çalıştığı güç ve fan hedefleriyle yarış durumuna (race condition) girerek donanımı kararsız kılabilir.

## Alternatifler
- **Alternatif A (Subprocess Sarmalama):** `std::process::Command` ile `ryzenadj` veya `intel-undervolt` komutlarını çağırmak. (Dış bağımlılık yükü, yavaş IPC ve hata yakalama zorluğu sebebiyle reddedildi).
- **Alternatif B (Sadece ACPI Profilleri ile Yetinmek):** Voltaj ve SMU ayarlarını tamamen terk edip yalnızca BIOS thermal modlarını kullanmak. (Oyun ve yüksek performans tutkunları için yetersiz bulunarak reddedildi).
- **Alternatif C (Doğrudan Rust İçi Bit-Packing & SMU Sürücüsü):** Donanım MSR yazmaçlarına ve AMD SMU posta kutusuna doğrudan Rust içinden yerel erişim sağlamak ve çakışma dedektörü ile korumak.

## Karar
Sistem Intel ve AMD platformları için tamamen **Yerel (Native) Donanım İnce Ayar Mimarisi** geliştirmiştir:
1. **Intel MSR Doğrudan Erişim:** [[cpu-msr-undervolt]] modülünde `/dev/cpu/0/msr` aygıtı üzerinden MSR `0x150` (Voltaj Düzlemleri) ve MSR `0x1A2` (Sıcaklık ve TCC) yazmaçlarına doğrudan 64-bit bayt paketleme (`pack_offset`, `convert_offset`) mantığı Rust ile yazılmıştır.
2. **AMD SMU Posta Kutusu:** [[amd-ryzen-smu]] modülü; Zen1+, Renoir, Cezanne, Rembrandt, Phoenix, HawkPoint, StrixPoint ve Raphael dahil tüm AMD mikro mimarilerini DMI ve CPUID ile tespit eder. SMU komutlarını doğrudan bellek eşlemeli yazmaçlar veya MSR posta kutusu kanalıyla gönderir.
3. **Çakışma Önleme Kalkanı:** [[conflict-detector]] servisi sistem başlangıcında `nbfc`, `throttled`, `ryzenadj` gibi çakışan süreçleri denetler; çakışma varsa kullanıcıyı bilgilendirerek donanım yarışlarını önler.
4. **D-Bus Servis Ayrımı:** Kontroller [[undervolt-service]] ve [[ryzen-service]] olmak üzere iki bağımsız D-Bus arabirimi üzerinden [[interfaces/undervolt-dbus-interface]] ve [[interfaces/ryzen-dbus-interface]] sözleşmelerine bağlanmıştır.

## Sonuçlar
### Olumlu
- **Sıfır Dış Bağımlılık:** Sistem harici hiçbir yardımcı ikili dosyaya ihtiyaç duymadan saf Rust ile voltaj ve SMU kontrolü sağlar.
- **Mikrosaniye Düzeyinde Tepki:** Subprocess çağırma gecikmesi ortadan kalkmış, doğrudan yazmaç erişimiyle anında donanım tepkisi elde edilmiştir.
- **Katı Tip ve Sınır Güvenliği:** Voltaj ofsetleri yazılımsal olarak güvenli aralıklara (örn. maks -150mV) kilitlenerek aşırı voltaj düşürme kaynaklı anlık donanım kilitlenmeleri (BSOD/Kernel Panic) yazılım katmanında sınırlandırılmıştır.

### Olumsuz / Trade-off
- Çekirdek seviyesinde `msr.allow_writes=on` parametresinin etkin olması veya daemon'ın `/dev/cpu/*/msr` üzerinde okuma/yazma hakkına sahip olması zorunludur.
- Intel 12. nesil sonrası bazı kilitli BIOS'larda undervolt MSR yazımları donanımsal olarak göz ardı edilebilir (Undervolt Protection).

## İlgili Bağlantılar
- Intel Donanım Modülü: [[cpu-msr-undervolt]]
- AMD Donanım Modülü: [[amd-ryzen-smu]]
- Servis Katmanı: [[undervolt-service]], [[ryzen-service]]
- Çakışma Denetimi: [[conflict-detector]]
