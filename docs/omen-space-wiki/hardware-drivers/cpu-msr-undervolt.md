# Intel CPU MSR Doğrudan Voltaj ve Sıcaklık İnce Ayarı

## Genel Bakış
Intel Core işlemcilerde voltaj düşürme (undervolting) ve termal tavanı kısıtlama (TCC offset), Model-Specific Register (MSR) adı verilen donanımsal kayıt yazmaçları üzerinden gerçekleştirilir.

OMEN Space (`src/omen-space-daemon/src/undervolt.rs`), harici hiçbir CLI aracı (örneğin `intel-undervolt`) kullanmadan doğrudan `/dev/cpu/0/msr` aygıtına 64-bit hassasiyette ikili veri yazarak voltaj ofsetlerini uygular.

## MSR Kayıt Haritası ve Düzlem Mimarisi

İki temel MSR yazmacı yönetilmektedir:
1. **Voltaj Ofset Kaydı (`0x150`):** `MSR_VOLTAGE_OFFSETS`
2. **Sıcaklık ve Termal Denetim Kaydı (`0x1A2`):** `MSR_TEMPERATURE`

### Voltaj Düzlemleri (Voltage Planes)
İşlemci mimarisinde voltaj değerleri birbirinden bağımsız 5 farklı donanım düzlemine ayrılmıştır:

| Düzlem İndeksi | Düzlem Adı (`plane`) | Açıklama |
| :---: | :--- | :--- |
| `0` | `core` | Ana CPU işlem çekirdekleri |
| `1` | `gpu` | Dahili Intel HD/Iris grafik birimi (iGPU) |
| `2` | `cache` | L3 CPU önbellek ve halka veri yolu (Ring Bus) |
| `3` | `uncore` | Sistem Ajanı (Bellek denetleyicisi ve I/O) |
| `4` | `analogio` | Analog giriş/çıkış ve faz kilitli döngü (PLL) |

## Bit Paketleme Matematiği (Bit-Packing Logic)

Intel MSR `0x150` yazmacına veri yazılırken ve okunurken 64-bitlik katı bir protokol izlenir:

```
[63]       : Yazma Tetikleyici Biti (1 = Write)
[43:40]    : Voltaj Düzlem İndeksi (Plane Index: 0..4)
[36]       : Voltaj Ofset Modu Seçimi (1)
[32]       : Sabit Başlatma Biti (1)
[31:21]    : 11-bit İki'ye Tümleyen Voltaj Ofseti (mV * 1.024)
[20:0]     : 0 (Ayrılmış / Sıfırlanmış)
```

### Dönüşüm Formülü
`undervolt.rs` içerisindeki dönüştürücü mantık:
```rust
fn convert_offset(mv: i32) -> u64 {
    let rounded = (mv as f64 * 1.024).round() as i32;
    0xFFE00000u64 & (((rounded as u64) & 0xFFF) << 21)
}

fn pack_offset(plane: u32, offset_bits: u64) -> u64 {
    (1u64 << 63)
        | ((plane as u64) << 40)
        | (1u64 << 36)
        | (1u64 << 32)
        | (offset_bits & 0xFFE00000u64)
}
```

## TCC (Thermal Control Circuit) Ofset Yönetimi
MSR `0x1A2` yazmacının 24 ile 29. bitleri arasındaki TCC ofseti, işlemcinin maksimum kritik sıcaklık tavanını (TjMax, örneğin 100°C) donanımsal olarak aşağı çeker. Örneğin 10°C ofset girildiğinde işlemci 90°C'ye ulaştığı anda termal kısmaya (throttling) girerek aşırı ısınmayı önler.

## İlgili Bağlantılar
- Mimari Karar: [[adr-004-native-msr-and-smu-mailbox-tuning]]
- Servis Katmanı: [[undervolt-service]]
- D-Bus Sözleşmesi: [[undervolt-dbus-interface]]
- Donanım İzinleri: [[udev-device-rules]]
