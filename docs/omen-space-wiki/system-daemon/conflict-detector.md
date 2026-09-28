# Donanım Çakışma Dedektörü (Conflict Detector)

## Genel Bakış
`ConflictDetector` (`src/omen-space-daemon/src/conflict_detector.rs`), sistemde OMEN Space ile aynı donanım yazmaçlarına (EC, MSR, SMU, WMI) müdahale edebilecek rakip süreçleri tespit eden koruma modülüdür.

Daemon başlatıldığında ilk olarak bu modül devreye girer ve olası yarış durumlarını (race conditions) raporlar.

## Bilinen Çakışan Süreçler Kataloğu

| Süreç Adı | Tehlike Seviyesi | Olası Çakışma Nedeni |
| :--- | :---: | :--- |
| `nbfc` | Yüksek | Notebook FanControl servisi, HP WMI ve EC fan kayıtlarına müdahale ederek [[fan-service]] ile döngüsel yarışa girer. |
| `throttled` | Yüksek | Lenovo/Generic throttled servisi, Intel MSR ve RAPL limitlerini ezerek termal profilleri bozar. |
| `ryzenadj` | Yüksek | Arka planda çalışan ryzenadj döngüsü, AMD SMU posta kutusu yazımlarında [[ryzen-service]] ile çakışır. |
| `hp-health` | Orta | Eski resmi HP sistem durumu arka plan servisi. |
| `oghaagent` | Orta | Wine veya Proton altında çalışan HP OMEN Gaming Hub ajanının gereksiz kaynak tüketimi ve kancaları. |

## Tespit Mantığı ve `ConflictReport`
Modül, `/proc` dizinini tarayarak çalışan aktif süreç listesini alır:
```rust
pub struct ConflictReport {
    pub has_conflicts: bool,
    pub conflicting_processes: Vec<String>,
    pub warning_message: String,
}
```
Eğer çakışan bir süreç tespit edilirse, sistem loglarına uyarı düşülür ve kullanıcı arayüzü teşhis sekmesinde uyarı kartı görüntülenir.

## İlgili Bağlantılar
- Mimari Yaşam Döngüsü: [[daemon-overview]]
- Fan Servisi: [[fan-service]]
- Ryzen Servisi: [[ryzen-service]]
- Mimari Karar: [[adr-004-native-msr-and-smu-mailbox-tuning]]
