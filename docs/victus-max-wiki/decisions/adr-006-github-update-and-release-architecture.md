# ADR-006: GitHub Entegrasyonlu OTA Güncelleme ve Sürüm Mimarisi

## Bağlam
Victus Max projesinin önceki güncelleme kontrol mimarisi, doğrudan GitHub Releases uç noktasına (`/releases/latest`) sorgu atıyordu. Ancak açık kaynaklı veya yeni çatallanmış depolarda resmi bir sürüm (release/tag) henüz yayınlanmamış olduğunda, GitHub API HTTP 404 (Not Found) durum kodu dönmekteydi. Bu durum grafik arayüzün (GUI) "No release found" hatasıyla takılmasına ve kullanıcının güncelleme merkezinden faydalanamamasına yol açıyordu.

Ayrıca sistemde aşağıdaki kritik mimari eksiklikler bulunmaktaydı:
1. **Kanal Ayrımı ve Tercih Eksikliği:** Kullanıcının en kararlı resmi sürümleri mi (Stable) yoksa ana geliştirme dalındaki (`main`) en yeni özellikleri ve hata düzeltmelerini mi (Canary) takip etmek istediğini belirleyebileceği bir kanal mimarisi bulunmuyordu.
2. **Kalıcı Ayar Yokluğu:** Arayüzdeki tercihler oturum kapandığında kayboluyor, `settings.json` üzerinde kalıcı bir `update_channel` alanı yönetilmiyordu.
3. **Eksik Güncelleme Betiği ve Yetki Sorunu:** Grafik arayüz unprivileged (ayrıcalıksız) kullanıcı alanında çalışırken ([[adr-001-rust-daemon-client-split]]), sistem bileşenlerini güncellemek için `/usr/share/victus-max/setup.sh` yolunu doğrudan `pkexec` ile çalıştırmayı deniyordu. Ancak bu dosya standart sistem paketlemesinde `/usr/share/` altında yer almıyordu ve özel bir Polkit yetkilendirme kuralı tanımlanmadığı için yetki yükseltme başarısız oluyordu.
4. **Yüksek Kaynak Tüketimi ve Derleme Yükü:** Güncelleme her seferinde sıfırdan `cargo build --release` ile kaynaktan derleme yapmaya çalışıyor, bu da 3-5 dakikalık bekleme süresine ve yüksek işlemci yüküne yol açıyordu; önceden derlenmiş ikili paketleri (prebuilt binary artifacts) dağıtacak bir CI/CD otomasyonu bulunmuyordu.

## Alternatifler

1. **Yalnızca Yerel Git Pull ve Kaynaktan Derleme:**
   - Kullanıcıdan terminalde `git pull && sudo ./setup.sh update` komutunu çalıştırmasını istemek veya GUI'den bunu tetiklemek.
   - *Değerlendirme:* Son kullanıcı dostu değildir; terminal bağımlılığı yaratır. Resmi kararlı sürümler ile geliştirme commit'leri arasında ayrım yapamaz; her güncellemede yüksek derleme süresi ve kaynak tüketimi gerektirir. Reddedildi.

2. **Yalnızca GitHub Releases Takibi:**
   - Yalnızca GitHub Releases REST API uç noktasını kullanmak.
   - *Değerlendirme:* Depoda henüz yayınlanmış bir GitHub Release bulunmadığında API 404 döner ve güncelleme mekanizması tamamen çöker. Kullanıcılar `main` dalındaki acil hata düzeltmelerine ve yeni donanım yamalarına erişemez. Reddedildi.

3. **Hibrit Çift Kanallı OTA ve Kalıcı Tercih Mimarisi (Seçilen):**
   - Canary (Git `main` dalı commit'leri) ve Stable (GitHub Releases) çift kanal yapısı.
   - Kullanıcının kanal tercihi `~/.config/victus-max/settings.json` içerisinde kalıcı olarak saklanır.
   - Sıfır-404 dayanıklılığı: Stable kanalda release bulunamadığında sistem otomatik olarak Canary kanalına geri çekilir (fallback) ve AdwBanner ile durumu kullanıcıya açıklar.
   - Güvenli yetki ayrımı: `/usr/libexec/victus-max/victus-max-updater` betiği ve `org.hp.victusmax.update` Polkit eylem politikası ile güvenli root çalıştırma.
   - Otomasyon ve paketleme: GitHub Actions `release.yml`, sürüm hazırlama aracı `scripts/release.sh` ve `setup.sh` içerisinde önceden derlenmiş ikili paket (`dist/bin/`) desteği.

## Karar

1. **Çift Kanallı Güncelleme Mantığı (Canary vs Stable):**
   - `UpdateChannel` modeli (`Canary`, `Stable`) tanımlandı (`src/victus-max-gui/src/update_checker.rs`).
   - **Stable Kanal:** `https://api.github.com/repos/umutaktepe/victus-max/releases/latest` uç noktasından son yayınlanan etiketi çeker ve SemVer karşılaştırması (`is_newer_semver`) ile yeni bir sürüm olup olmadığını doğrular.
   - **Canary Kanal:** `https://api.github.com/repos/umutaktepe/victus-max/commits/main` uç noktasından en son commit SHA özetini çeker ve derleme anında gömülen `VICTUS_MAX_GIT_HASH` ile karşılaştırır.
2. **Kalıcı Kullanıcı Tercihi:**
   - Kullanıcının kanal tercihi `~/.config/victus-max/settings.json` (ve geriye dönük fallback olarak `~/.config/omenspace/settings.json`) dosyasında `update_channel` alanı altında kalıcı hale getirildi.
   - GUI başlatıldığında `load_update_channel()` ile ayar okunur; kullanıcı arayüzdeki `AdwComboRow` üzerinden değişiklik yaptığında `save_update_channel()` anında diske kaydeder.
3. **Hata Toleransı ve Sıfır-404 Fallback:**
   - Stable kanal sorgusu sırasında GitHub API 404 (Not Found) yanıtı döndüğünde (depoda henüz sürüm yayınlanmamışsa), güncelleyici hata fırlatmak yerine otomatik olarak Canary kanalını sorgular (`fallback_to_canary = true`).
   - GUI tarafında turuncu bir AdwBanner görüntülenerek kullanıcıya henüz kararlı sürüm bulunmadığı, bu nedenle en güncel geliştirme sürümünün gösterildiği şeffafça bildirilir.
4. **Yetkilendirme ve Güvenlik:**
   - GUI unprivileged kullanıcı oturumunda çalışır ([[adr-001-rust-daemon-client-split]]).
   - Güncelleme yürütmesi bağımsız `/usr/libexec/victus-max/victus-max-updater` betiğine devredilmiştir.
   - Polkit kuralı `data/org.hp.victusmax.update.policy` ile `org.hp.victusmax.update` eylemi tanımlanmış ve `/usr/share/polkit-1/actions/` altına kurulmuştur.
   - GUI, `pkexec /usr/libexec/victus-max/victus-max-updater <channel>` komutunu çağırarak güvenli root yetkilendirme diyaloğunu açar.
5. **Aşamalı İlerleme Takibi (Stage Tags):**
   - Güncelleyici betik standart çıktıya standart aşama etiketleri basar:
     - `[STAGE:PREPARE]` — Çalışma ortamı ve bağımlılık kontrolü (%15)
     - `[STAGE:DOWNLOAD]` — Kaynak kod veya arşiv indirme (%35)
     - `[STAGE:BUILD]` — Bileşen derleme veya prebuilt doğrulama (%70)
     - `[STAGE:INSTALL]` — Sistem dosyalarını kopyalama (%85)
     - `[STAGE:RESTART]` — Servisleri yeniden başlatma (%95)
     - `[STAGE:COMPLETE]` — Güncelleme tamamlandı (%100)
   - GTK4 GUI bu etiketleri canlı olarak yakalayarak ilerleme çubuğunu (`ProgressBar`) günceller ve terminal çıktısını genişletilebilir terminal görünümünde sunar.
6. **Sürüm Otomasyonu ve Paketleme:**
   - GitHub Actions `.github/workflows/release.yml` iş akışı `v*.*.*` git etiketlerinde otomatik olarak çalışarak Ubuntu üzerinde ikili dosyaları derler ve `tar.gz` arşivi olarak GitHub Releases'a ekler.
   - `scripts/release.sh` betiği tüm `Cargo.toml` dosyalarında sürüm numaralarını günceller, git etiketini basar ve release sürecini standartlaştırır.
   - `setup.sh` betiği `dist/bin/` dizini algıladığında derleme aşamasını atlayarak ikili dosyaları doğrudan kurar; böylece güncelleme süresi saniyelere iner.

## Sonuçlar (Trade-offs)

### Olumlu Sonuçlar:
- **Kesintisiz Güncelleme Deneyimi:** Sıfır-404 dayanıklılığı sayesinde henüz GitHub Release yayınlanmamış olsa dahi güncelleme merkezi hatasız çalışır.
- **Güvenli Yetki Ayrımı:** Arayüzün root yetkisiyle çalışmasına gerek kalmadan, Polkit üzerinden sıkı denetimli ve kullanıcı onaylı sistem güncellemesi yapılır.
- **Kalıcı Kanal Tercihi:** Kullanıcının Canary veya Stable seçimi diske yazılarak her oturumda hatırlanır.
- **Şeffaf İlerleme:** Canlı aşama etiketleri sayesinde kullanıcı güncellemenin donduğunu sanmaz; adım adım hangi aşamada olduğunu ve terminal akışını izleyebilir.
- **Hızlı ve Esnek Kurulum:** Prebuilt ikili paket desteği sayesinde derleme süresi ortadan kalkar; kaynak kodu derleme fallback'i korunur.

### Olumsuz Sonuçlar / Ödünleşimler:
- **GitHub API Hız Sınırı (Rate Limit):** GitHub REST API anonim (unauthenticated) istekler için IP başına saatte 60 istek sınırı uygular. Hız sınırı aşıldığında `UpdateCheckError::RateLimitExceeded` hatası kullanıcıya iletilir.
- **Prebuilt İkili Uyumluluğu:** GitHub Actions üzerinde derlenen ikililer belirli glibc sürümlerine bağımlıdır. Çok eski veya standart dışı dağıtımlarda sistem otomatik olarak kaynaktan derlemeye fallback yapar.

## İlgili Bağlantılar
- [[updater-service]]
- [[gui-application]]
- [[polkit-dbus-security]]
- [[adr-001-rust-daemon-client-split]]
