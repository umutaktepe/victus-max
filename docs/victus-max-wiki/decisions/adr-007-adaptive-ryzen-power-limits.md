# ADR-007: İşlemciye Duyarlı (Adaptive) AMD Ryzen Güç Limitleri ve Canlı PMTable Telemetrisi

## Durum
Kabul Edildi

## Tarih
2026-10-10

## Bağlam
HP Omen ve Victus dizüstü bilgisayarlarında hem Intel (RAPL / MSR) hem de AMD Ryzen (SMU / STAPM / PPT) işlemcili konfigürasyonlar bulunmaktadır. Mevcut mimaride:
1. Arka planda (`victus-max-daemon`), AMD işlemciler için `ryzenadj` üzerinden güç limitlerini (`--stapm-limit`, `--slow-limit`, `--fast-limit`) ve sıcaklık sınırını (`--tctl-temp`) uygulama yeteneği bulunsa da, kullanıcı arayüzü (`victus-max-gui`) Power Limits bölümünü yalnızca Intel RAPL terminolojisi (`PL1`, `PL2`, `TCC Offset`) ve sabit aralıklarla sunmaktaydı.
2. Linux çekirdeğindeki `intel_rapl` sürücüsü AMD Ryzen mobil işlemcilerde yalnızca enerji sayaçlarını (`energy_uj`) sunmakta, güç kısıtlama dosyalarını (`constraint_0_power_limit_uw`) oluşturmamaktadır. Bu nedenle AMD sistemlerde aktif donanım limitleri okunamamakta ve arayüzde yalnızca sabit varsayılan değerler (45W / 80W) gösterilmekteydi.
3. Farklı TDP sınıflarındaki AMD işlemciler (15W U-serisi, 35-54W HS/H-serisi, 55W+ HX-serisi) aynı sabit slider aralıklarını paylaştığında kullanıcı deneyimi ve donanım güvenliği zedelenmekteydi.

## Alternatifler
- **Alternatif A (Tek Tip Statik Arayüz):** Mevcut Intel etiketlerini koruyup arka planda AMD'ye çevirmeye devam etmek. (Kullanıcıda "sadece Intel destekleniyor" yanılgısı yarattığı ve canlı değerleri okuyamadığı için reddedildi).
- **Alternatif B (Yalnızca Konfigürasyon Dosyasından Okuma):** Donanımı hiç sorgulamadan sadece `/etc/victus-max/power.json` içindeki son kaydedilen değeri göstermek. (Sistem başlangıcında veya BIOS/EC tarafından değiştirilen gerçek donanım limitlerini yansıtmadığı için yetersiz bulundu).
- **Alternatif C (İşlemci TDP Sınıflandırıcısı + Canlı RyzenAdj PMTable Okuyucu + Adaptif GTK4 Arayüzü):** İşlemci modelini sınıflandırarak uygun STAPM/Fast PPT/Tctl aralıklarını belirlemek, `ryzenadj -i` çıktısından canlı PMTable limitlerini ayrıştırmak ve GTK4 arayüzünü işlemci mimarisine göre dinamik olarak dönüştürmek.

## Karar
**Alternatif C** benimsenmiştir:
1. **İşlemci Mimarisi ve TDP Sınıflandırıcısı (`cpu_tier.rs`):** İşlemci model dizgesi analiz edilerek `AmdUltraLow` (U-serisi), `AmdStandardHs` (HS/H-serisi), `AmdExtremeHx` (HX-serisi), `IntelStandard` ve `IntelExtreme` sınıfları ile her sınıfa özgü güvenli minimum, maksimum ve varsayılan güç/sıcaklık zarfları tanımlanmıştır.
2. **Serileştirilmiş Canlı PMTable Telemetri Ayrıştırıcısı ve `GetPowerLimitsInfo` Ayrımı:** `/dev/mem` üzerinden PCI SMN (`0x60`/`0x64`) ve MP1 SMU posta kutusuna (`0x3B10528`) erişen `ryzenadj` süreçlerinin eşzamanlı çalışarak SMU posta kutusunu kilitlemesini (`Unable to get MP1 SMU Obj`) önlemek amacıyla tüm `ryzenadj` çağrıları `cpu_tier::run_ryzenadj` altında süreç-geneli `RYZENADJ_MUTEX` ve 2 saniyelik TTL önbelleği ile serileştirilmiştir. Ayrıca 1.5 saniyede bir yoklanan `GetPowerProfile` metodundan `ryzenadj -i` tamamen çıkarılarak yalnızca Gelişmiş Güç sekmesi açıldığında (`connect_map`) çağrılan `GetPowerLimitsInfo` D-Bus metoduna taşınmıştır.
3. **Doğrudan Tctl Sıcaklık Kontrolü:** AMD Ryzen işlemcilerde Intel'in ters `100 - TCC Offset` mantığı yerine doğrudan `75°C - 100°C` hedef Tctl sıcaklık slider'ı sunulmuş ve [[undervolt-service]] her iki formatı da destekleyecek şekilde genişletilmiştir.
4. **Nesle Duyarlı Güç Hiyerarşisi ve Adaptif GTK4 Slider Senkronizasyonu (`stapm_fast_linked`):** AMD mobil SMU bellenim mimarisinde nesiller arası davranış farkı modellenmiştir (`is_zen4_or_newer_ryzen`):
   - **Zen 4 / Zen 5 (`stapm_fast_linked = true`):** Donanım SMU mimarisinde `--fast-limit` uygulandığında STAPM otomatik olarak Fast PPT değerine kilitlenmektedir (`STAPM = PPT Fast`). Bu nesillerde `slow <= fast` kuralı geçerlidir. [[gui-application]] üzerinde STAPM veya PPT Fast slider'ı değiştirildiğinde diğeri anında senkronize olur (`STAPM <-> PPT Fast`) ve `PPT Slow` değeri bu tavanı aşamayacak şekilde dinamik sınırlandırılır.
   - **Zen 2 / Zen 3 / Zen 3+ (`stapm_fast_linked = false`):** Donanım 3 ayrı bağımsız kademeyi destekler ve mutlak `STAPM <= PPT Slow <= PPT Fast` hiyerarşisi geçerlidir. Slider'lar bağımsız hareket eder ancak hiyerarşik sınırları korumak adına birbirini dinamik olarak iter/çeker.
   - **Daemon Normalizasyonu (`normalize_ryzen_limits`):** D-Bus veya GUI üzerinden gelen tüm limitler donanıma yazılmadan önce `cpu_tier::normalize_ryzen_limits` ile işlemci nesline göre normalize edilir; böylece tutarsız veya hiyerarşiyi bozan limitlerin SMU'ya gönderilmesi engellenir.
   - Gelişmiş Güç sayfası, AMD Ryzen tespit edildiğinde başlıklarını, açıklamalarını ve slider aralıklarını işlemci sınıfına göre uyarlar; her sekme geçişinde (`connect_map`) canlı değerleri yeniler ve `apply_all_tuning_sync` ile ayarları yarış koşulu olmadan sıralı uygular.

## Sonuçlar
### Olumlu
- **Gerçek Donanım Durumu ve Sıfır SMU Çakışması:** Arka plan profil yoklaması SMU posta kutusuna hiç dokunmaz; Gelişmiş Güç sekmesi açıldığında gerçek `STAPM`, `PPT Slow`, `PPT Fast` ve `Tctl` limitleri güvenli ve serileştirilmiş biçimde okunur.
- **Mimari Netlik:** AMD kullanıcıları kendi platformlarına ait doğru mühendislik terimlerini (`STAPM`, `PPT Slow`, `PPT Fast`, `Tctl`) ve işlemcilerine uygun TDP aralıklarını görür.
- **Tam Geriye Dönük Uyumluluk:** Intel 12/13/14. nesil RAPL ve MSR iş akışları hiçbir kırılma olmadan korunmuştur; eski Intel varsayılanlarını (`45W / 80W`) içeren `power.json` dosyaları AMD sistemlerde otomatik göç ettirilerek başlangıçta donanım limitlerini ezmesi engellenmiştir.

### Olumsuz / Trade-off
- `GetPowerLimitsInfo` çağrısında (önbellek süresi dolmuşsa) `ryzenadj -i` alt süreci en fazla 1200ms zaman aşımıyla çalışır; bu çağrı GUI'de yalnızca Gelişmiş Güç sekmesi görüntülendiğinde asenkron (`glib::spawn_future_local`) yürütüldüğü için arayüz akıcılığını etkilemez.

## İlgili Bağlantılar
- Önceki Mimari Karar: [[adr-004-native-msr-and-smu-mailbox-tuning]]
- Donanım Katmanı: [[amd-ryzen-smu]], [[cpu-msr-undervolt]]
- Servis Katmanı: [[power-service]], [[undervolt-service]], [[ryzen-service]]
- D-Bus Sözleşmesi: [[power-dbus-interface]]
- Kullanıcı Arayüzü: [[gui-application]]
