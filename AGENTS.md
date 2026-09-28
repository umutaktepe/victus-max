# AGENTS.md — OMEN Space Geliştirici ve Ajan Kuralları (LLM Wiki Sözleşmesi)

Bu belge, bu kod tabanında (`omen-space`) geliştirme yapacak, kod tabanını değiştirecek, yeni özellik ekleyecek veya dokümantasyonu sürdürecek **tüm yapay zeka ajanları ve mühendisler** için bağlayıcı kural setidir.

Bu depo, **Andrej Karpathy'nin "LLM Wiki / Living Architecture"** paradigmasına göre yönetilmektedir.

---

## 1. Temel Prensipler ve Kırılmaz Kurallar (Iron Rules)

### Kural 1: Sayısal/Kitap Düzeni Kesinlikle Yasaktır
- `01-giris.md`, `02-kurulum.md`, `03-moduller.md` gibi basılı kitap düzeni taklidi sıralı dosya ve klasör isimleri **KESİNLİKLE KULLANILAMAZ**.
- Bilgi tabanı sıralı bir kılavuz değil; yaşayan, dinamik, ilişkisel bir mühendislik ağıdır (graph/associative memory).

### Kural 2: Fonksiyonel Domain Ayrımı (No Numbers, Only Roles)
- Tüm klasörler numaralarla değil, yerine getirdiği mimari ve mühendislik rolleriyle adlandırılmalıdır:
  - `decisions/`: Mimari Karar Kayıtları (ADR).
  - `hardware-drivers/`: Çekirdek sürücüleri, ACPI, EC, MSR ve SMU.
  - `system-daemon/`: Root arka plan mikroservisleri ve motorlar.
  - `interfaces/`: D-Bus IPC sözleşmeleri, telemetri şemaları ve donanım matrisleri.
  - `user-experience/`: Kullanıcı alanı arayüzleri (GTK4, Overlay HUD, CLI, Tray).
  - `platform-integration/`: Systemd, Polkit, Udev ve paketleme.

### Kural 3: İki Yönlü Wikilink ve Graf Hijyeni (Cluster Topology)
- Dokümantasyon içerisinde geleneksel Markdown linkleri (`[metin](yol.md)`) yerine **Obsidian uyumlu `[[dosya-adi]]`** sözdizimi kullanılmalıdır (uzantı `.md` yazılmaz).
- **Hairball Yasağı:** Her sayfanın rastgele her sayfaya bağlandığı anlamsız düğüm yumaklarından kaçınılmalıdır.
- Bir sayfa yalnızca doğrudan bağımlı olduğu mimari karara (`[[adr-...]]`), veri şemasına veya donanım sürücüsüne bağlanmalıdır. Bağlantılar cümle içinde doğal bir bağlamla kurulmalıdır.

### Kural 4: Yaşayan Bellek ve Zorunlu ADR (Architectural Decision Records)
- Kod tabanında yapılan herhangi bir kritik mimari değişiklik (yeni kütüphane seçimi, veri formatı değişikliği, donanım register yaklaşımı, güvenlik politikası vb.) için `docs/omen-space-wiki/decisions/` altında yeni bir `adr-00x-[konu].md` oluşturulması **ZORUNLUDUR**.
- Her ADR; **Bağlam**, **Alternatifler**, **Karar**, **Sonuçlar (Olumlu/Olumsuz Trade-off)** ve **İlgili Bağlantılar** bölümlerini eksiksiz içermelidir.

### Kural 5: Adlandırma Standartları (Kebab-Case)
- Wiki altındaki tüm klasör ve dosya isimleri küçük harfli ve tireli (`kebab-case`) olmalıdır (Örn: `fan-curve-editor-ui.md`).

---

## 2. Ajan İş Akışları (Agent Workflows)

### Yeni Bir Özellik veya Kod Değişikliği Yaparken:
1. **İncele:** İlgili domain altındaki wiki sayfalarını ve ilgili `[[adr-...]]` kayıtlarını oku.
2. **Kodu Yaz:** Rust / C değişikliklerini tamamla ve doğrula.
3. **Wiki'yi Güncelle:**
   - Değişiklikten etkilenen mevcut atomik `.md` sayfasını güncelle.
   - Eğer yeni bir kavram veya servis eklendiyse uygun domain klasöründe yeni atomik sayfa aç.
   - `docs/omen-space-wiki/index.md` (MOC) dosyasına yeni sayfayı tek satırlık özetle ekle.
4. **Log Düş (Günlük Tut):**
   - `docs/omen-space-wiki/log.md` dosyasının sonuna formatına uygun olarak append-only kayıt ekle:
     `## [YYYY-MM-DD] İşlem_Türü | Başlık ve kısa özet`

### Bilgi Tabanını Denetlerken (Linting Pass):
- Sayfalar arasında kopuk link (dangling wikilink) olup olmadığını kontrol et.
- Kod ile belgelendirme arasında uyumsuzluk olup olmadığını tara.
- Yeni eklenen bir anakart varsa `boards.json` ve `board-capabilities-matrix.md` dokümanını güncelle.

---

## 3. Dizin Yapısı Referansı

```
docs/omen-space-wiki/
├── index.md                 # Ana İçerik Haritası (MOC)
├── log.md                   # Kronolojik İşlem Günlüğü
├── decisions/               # Mimari Karar Kayıtları (ADR-001, ADR-002, ...)
├── hardware-drivers/        # Çekirdek, WMI, EC, MSR, SMU
├── system-daemon/           # Daemon, fan, power, rgb, mux servisleri
├── interfaces/              # D-Bus IPC sözleşmeleri, JSON şemaları
├── user-experience/         # GTK4 GUI, HUD Overlay, Tray, CLI
└── platform-integration/    # Systemd, Polkit, Udev, Dağıtım
```
