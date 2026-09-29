# Intel Undervolt ve Termal Kısıtlama Servisi

## Genel Bakış
`UndervoltService` (`src/victus-max-daemon/src/undervolt.rs`), Intel işlemcili HP Omen ve Victus modellerinde çekirdek voltajını düşürerek termal kısmayı (thermal throttling) önleyen ve enerji verimliliğini artıran sistem servisidir.

Servis, D-Bus üzerinde `org.hp.omen.Undervolt` arayüzünü ([[undervolt-dbus-interface]]) sunar ve donanım yazmaçlarına doğrudan [[cpu-msr-undervolt]] modülü üzerinden erişir.

## Mimari Yetenekler

### 1. Çok Düzlemli Voltaj Yönetimi (Multi-Plane Offsets)
Servis, Intel mimarisindeki 5 bağımsız voltaj düzlemine negatif ofset uygulayabilir:
- `core`: CPU çekirdek voltajı.
- `cache`: L3 önbellek / ring bus voltajı (Genellikle çekirdek ile dengeli tutulmalıdır).
- `gpu`: Entegre Intel grafik yongası voltajı.
- `uncore`: Sistem Ajanı voltajı.
- `analogio`: Giriş/çıkış bileşenleri voltajı.

### 2. Canlı MSR Okuma ve Doğrulama (`ReadOffsets`)
Servis yalnızca kayıtlı konfigürasyonu raporlamakla kalmaz, `/dev/cpu/0/msr` üzerindeki `0x150` yazmacından doğrudan canlı veri okuyarak uygulanan voltajın BIOS tarafından kabul edilip edilmediğini doğrular.

### 3. TCC Sıcaklık Tavanı Ayarı (`SetTccOffset`)
İşlemcinin kritik termal tavanını (TjMax) MSR `0x1A2` üzerinden aşağı çekerek sistemin belirlenen sıcaklığın (örn. 85°C) üzerine çıkmasını engeller.

### 4. Güvenlik Korumaları ve Sınırlar
Aşırı voltaj düşürme kaynaklı sistem çökmelerini önlemek amacıyla yazılımsal güvenlik limitleri (örn. maksimum -150mV) tanımlanmıştır. Tanımlı limitlerin dışındaki D-Bus istekleri reddedilir.

### 5. Kalıcılık
Yapılandırılan voltaj ve TCC değerleri `/etc/victus-max/undervolt.json` dosyasında saklanır (eski `/etc/omen-space/undervolt.json` geriye dönük fallback desteğiyle) ve sistem her açıldığında servis tarafından otomatik olarak MSR yazmaçlarına yazılır.

## İlgili Bağlantılar
- Donanım Sürücüsü: [[cpu-msr-undervolt]]
- D-Bus Sözleşmesi: [[undervolt-dbus-interface]]
- Mimari Karar: [[adr-004-native-msr-and-smu-mailbox-tuning]]
- Arayüz Kontrolü: [[gui-application]]
