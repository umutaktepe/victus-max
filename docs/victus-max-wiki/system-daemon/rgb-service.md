# RGB Aydınlatma ve Efekt Servisi

## Genel Bakış
`RgbService` (`src/victus-max-daemon/src/rgb/mod.rs`), HP Omen ve Victus serisi dizüstü ve masaüstü bilgisayarlarda yer alan klavye, logo ve kasa aydınlatma donanımlarını yöneten sistem servisidir.

Servis, D-Bus üzerinde `org.hp.omen.Rgb` arayüzünü ([[rgb-dbus-interface]]) sunar ve alt katmanda [[hp-omen-extra-dkms]] kernel sürücüsü veya USB HID aygıtları ile haberleşir.

## Donanım Mimarisi ve Algılama

`RgbHardware` yapısı, sistem başlangıcında donanım topolojisini tespit eder:
- **4 Bölgeli (4-Zone) Klavyeler:** Klavye sol, orta-sol, orta-sağ ve sağ olmak üzere 4 bölgeye ayrılmıştır. Çoğu Victus ve standart Omen 15/16 modeli bu kategoridedir.
- **Tuş Başına (Per-Key) RGB:** Her bir tuşun bağımsız RGB LED'e sahip olduğu üst düzey modeller (Omen 16/17 Transcend).
- **Işık Çubuğu (Lightbar):** Kasa önünde veya menteşe altında bulunan harici RGB şeridi.
- **Masaüstü (Desktop RGB):** HP Omen 25L/40L/45L masaüstü kasalarında bulunan fan ve RAM aydınlatmaları (`desktop_rgb.rs`).

## Desteklenen Aydınlatma Modları ve Animasyonlar

Statik ve dinamik efektler daemon içindeki asenkron zamanlayıcı döngüleri tarafından hesaplanır:

| Mod Adı | Açıklama |
| :--- | :--- |
| `static` | Bölgeler kullanıcı tarafından belirlenen sabit renkleri korur. |
| `breathing` | Seçili renkler arasında yumuşakça sönen ve parlayan nefes efekti. |
| `wave` | Soldan sağa (`ltr`) veya sağdan sola (`rtl`) hareket eden dalga animasyonu. |
| `cycle` | Tüm RGB spektrumunu sırayla dönen renk döngüsü. |
| `rainbow` | Çok bölgeli yumuşak gökkuşağı geçişi. |
| `pulse` / `chase` | Hızlı atan kalp atışı ve kovalama efektleri. |
| `sparkle` / `disco` | Rastgele tuş/bölge parlamaları ve disko aydınlatması. |

## Tuş Başına Eşleme Sihirbazı (Per-Key HID Wizard)
Tuş başına aydınlatmalı modellerde Linux çekirdeğinin tuş sıralaması ile donanım indeksleri farklılık gösterebilir. Servis; `start_per_key_wizard()`, `light_key_index()` ve `record_key_mapping()` metotlarıyla kullanıcının tuşları tek tek yakıp haritalamasını sağlayan interaktif bir eşleme motoru barındırır. Bu harita `~/.config/victus-max/per_key_map.json` (veya `/etc/victus-max/per_key_map.json`) dosyasına yazılır.

## Yapılandırma Kalıcılığı
Uygulanan renk kodları, parlaklık seviyeleri ve animasyon hızları `/etc/victus-max/rgb.json` dosyasında saklanır (eski `/etc/omen-space/rgb.json` geriye dönük fallback desteğiyle) ve sistem açılışında otomatik olarak geri yüklenir.

## İlgili Bağlantılar
- Donanım Sürücüsü: [[hp-omen-extra-dkms]]
- D-Bus Sözleşmesi: [[rgb-dbus-interface]]
- Mimari Karar: [[adr-003-companion-dkms-driver-non-clashing]]
- Grafik Editör: [[gui-application]]
