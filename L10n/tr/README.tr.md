[English](../../README.md) | [简体中文](../zh-cn/README_zh-CN.md) | [Español](../es/README_es.md) | [Português (BR)](../pt-BR/README_pt-BR.md) | [日本語](../ja/README_ja.md) | [Русский](../ru/README_ru.md) | [Français](../fr/README_fr.md) | [Deutsch](../de/README_de.md) | [한국어](../ko/README_ko.md) | **Türkçe** | [Bahasa Indonesia](../id/README.id.md)

# FamTree

**[john3sobieski](https://github.com/john3sobieski)** tarafından oluşturuldu ve sürdürülüyor. Depo: <https://github.com/john3sobieski/family-tree>

Aile ağacınızı oluşturmak, düzenlemek ve incelemek için bir Windows masaüstü uygulaması ve ağacı telefonda görüntülemek için bir Android uygulaması. Her şey tek bir `.famtree` dosyasında durur: kişiler, ilişkiler ve tüm fotoğraflar.

> **Durum:** kullanıma hazır. Proje tamamlanmış sayılır: bundan sonra yalnızca küçük düzeltmeler beklenir. Windows yükleyicisini veya Android APK'sını [Releases](https://github.com/john3sobieski/family-tree/releases) sayfasından indirin ([Kurulum](#kurulum) bölümüne bakın).

## Özellikler

- **Üç mod**
  - **Görüntüleme** (*View*) – akıcı yakınlaştırma ve kaydırma sunan, salt okunur zihin haritası.
  - **Düzenleme** (*Edit*) – kişi, fotoğraf ve ilişki ekleyin ve değiştirin.
  - **Liste** (*List*) – ağaçtaki herkesin sıralanabilir ve filtrelenebilir tablosu.
- **Sizin etrafınızda kurulan zihin haritası.** Bir kişiyi "ben" olarak işaretleyin; ağaç en üstteki atalarınızdan aşağıya doğru çizilir. Böylece anne-baba, kardeşler, amca/dayı, hala/teyze, kuzenler ve daha uzak akrabalar otomatik olarak görünür. Ailenin her iki tarafı yan yana yerleştirilir, evlilik yoluyla akraba olunan aileler de dahil edilir.
- **Fotoğraflı kartlar.** Her kart vesikalık tarzı bir fotoğraf, ad, soyadı (kadınlarda kızlık soyadı parantez içinde), baba adı (patronimik), cinsiyeti gösteren renkli bir şerit ve en fazla iki uyruk bayrağı gösterir.
- **Kişi penceresi.** Bir karta tıklamak; tam fotoğraf, galeri (kişi başına 10+ fotoğraf sorun değil), tarihler, yaş, doğum ve ölüm yerleri, meslek, notlar ve anne-baba, partnerler ve çocuklara tıklanabilir bağlantılar içeren daha büyük bir pencere açar.
- **Arama.** Bir ad yazın; kamera haritada o kişiye uçar ve kişiyi vurgular. Liste modunda filtre olarak çalışır. `Ctrl+F` arama alanına odaklanır.
- **İlişkiler.** Anne-baba, partnerler, çocuklar ve kardeşler tek tıkla eklenebilir veya zaten mevcut kişilere bağlanabilir. Döngüler ve yinelenen bağlantılar reddedilir.
- **Eksik tarihler.** Yalnızca yılı mı biliyorsunuz? Sorun değil, bir tarihin herhangi bir kısmı boş bırakılabilir.
- **Uyruklar.** Kişi başına, 197 ülkelik yerleşik bir listeden (tüm BM üyeleri artı Vatikan, Filistin, Tayvan ve Kosova) en fazla iki uyruk seçin. Bayrakları uygulamanın kendisi çizer.
- **Büyük ağaçlar için tasarlandı.** Yalnızca görünen kartlar çizilir, küçük resimler tembel yükleme (lazy loading) ile yüklenir ve yerleşim yalnızca yapı değiştiğinde yeniden hesaplanır. Binlerce kişilik ağaçlarla test edilmiştir.
- **Otomatik kaydetme.** Ağacın bir dosyası olduktan sonra, yazmayı bıraktıktan kısa bir süre sonra her değişiklik otomatik olarak kaydedilir. Yeni bir ağaç için önce **Farklı kaydet…** (*Save as…*) seçeneğini kullanın. Bir şey kaydedilemediyse kapatmadan önce size sorulur.
- **Arayüz dilleri.** Русский, English, 中文, Español, Français, Deutsch ve Polski. Dili **Language** menüsünden değiştirebilirsiniz; seçiminiz hatırlanır. İlk açılışta uygulama Windows görüntüleme dilini izler, bu dillerden biri değilse İngilizceyi kullanır. Ülke adları da çevrilir.
- **Dışa aktarma.** **File → Export…** tüm ağacı kenar boşluklarıyla PNG veya vektör PDF olarak kaydeder. PNG için arka plan (şeffaf, beyaz veya siyah) ve koyu ya da açık renkler seçilir; boyut her şeyin net kalması için otomatik belirlenir (en fazla ×2). PDF kalite kaybı olmadan yakınlaştırılabilir. Çok büyük ağaçları PDF olarak dışa aktarmak daha iyidir.
- **Çift tıklayarak açma.** **Dosya** (*File*) menüsünden `.famtree` dosyalarını uygulamayla ilişkilendirebilirsiniz.
- **Koyu ve açık tema.** Durum çubuğunun sağındaki güneş/ay düğmesiyle değiştirilir; seçim hatırlanır.
- **Android uygulaması (yalnızca görüntüleme).** Aynı `.famtree` dosyasını telefonunuzda veya tabletinizde açın. Harita bilgisayardakiyle birebir aynı görünür (aynı yerleşim, kartlar, bayraklar ve çizgiler); sürükleme ve iki parmakla yakınlaştırma, kameranın kişiye uçtuğu arama, fotoğraflar ve akraba bağlantılarıyla tam ekran kişi sayfası, *Fit to view* ve açık/koyu tema desteklenir. Dosya asla değiştirilmez: düzenleme, Liste modu ve dışa aktarma bilgisayarda kalır. Arayüz telefonun dilini izler (aynı yedi dil, aksi halde İngilizce). Bkz. [Kurulum → Android](#android).

## Kurulum

### Windows

Uygulama Windows 10 ve Windows 11'de çalışır. İkisi için de aynı yükleyici ve aynı exe kullanılır.

1. [Releases](https://github.com/john3sobieski/family-tree/releases) sayfasını açın ve **`FamTree-Setup-<sürüm>.exe`** dosyasını indirin.
2. Çalıştırın. Yalnızca kullanıcınız için kurulur, yönetici hakkı gerekmez. Sihirbaz, `.famtree` dosyalarının FamTree ile açılmasını ve masaüstü kısayolu eklenmesini önerir.
3. **FamTree**'yi Başlat menüsünden açın.

Notlar:

- Yükleyici ve `.exe` imzalı olmadığı için Windows bir SmartScreen uyarısı gösterebilir. *Daha fazla bilgi → Yine de çalıştır* seçeneğini seçin.
- Güncellemek için daha yeni bir Setup'ı eski sürümün üzerine çalıştırın. Ağaçlarınız ve ayarlarınız korunur.
- Kaldırmak için *Ayarlar → Uygulamalar* bölümünü kullanın. `.famtree` dosyalarınız asla silinmez.
- Taşınabilir exe'yi kullanıp taşırsanız, **Dosya** menüsündeki `.famtree` ilişkilendirme düğmesine yeniden basın.

### Android

Android uygulaması ağaçları yalnızca gösterir ve dosyanızı asla değiştirmez. 64 bit ARM'li bir cihazda Android 10 veya üstü gerekir (güncel telefon ve tabletlerin neredeyse tamamı).

1. **`FamTree-<sürüm>.apk`** dosyasını [Releases](https://github.com/john3sobieski/family-tree/releases) sayfasından doğrudan cihazınıza indirin.
2. Dosyayı açın. Android, bu kaynaktan (tarayıcınız veya dosya yöneticiniz) uygulama yüklenmesine izin vermenizi bir kez isteyecektir; onaylayın.
3. `.famtree` dosyanızı cihaza aktarın (bulut depolama, mesajlaşma uygulaması, USB kablosu). Ardından dosya yöneticisinde dosyayı FamTree ile açın (*Open with*) veya FamTree'yi başlatıp *Open* düğmesine dokunarak dosyayı seçin. En son açılan dosya hatırlanır. Uygulama çalışırken başka bir `.famtree` açarsanız, yeniden başlatmadan yüklenir.

Notlar:

- Uygulama yalnızca APK olarak dağıtılır, Google Play'de yoktur. Play Protect, bilinmeyen bir geliştiricinin uygulaması için uyarabilir; yine de yüklemeyi seçin.
- Yeniden açabilmek için uygulama, en son açılan dosyanın özel bir kopyasını tutar; orijinal dosya asla değiştirilmez.

### Kendiniz derleyin

Yükleyiciyi hiçbir araç kurmadan GitHub Actions'tan alabilirsiniz (bir GitHub hesabı gerekir): [Depoyu](https://github.com/john3sobieski/family-tree) fork edin, **Actions → Build → Run workflow** yolunu izleyin ve tamamlanan çalıştırmadan **`famtree-setup`** çıktısını indirin (varsayılan olarak 90 gün saklanır).

Android APK'sı da aynı şekilde elde edilir: **Actions → Android → Run workflow**, ardından **`famtree-android-<sürüm>`** çıktısını indirin.

## Hızlı başlangıç

1. `famtree.exe` dosyasını başlatın. Kendi ağacınızı başlatmak için **Dosya → Yeni** (*File → New*), mevcut bir ağacı açmak için **Dosya → Aç** (*File → Open*) seçeneğini kullanın.
2. **Düzenleme** moduna geçin ve ilk kişiyi ekleyin (*＋ New person*). Boş bir ağaçta bu kişi otomatik olarak "ben" olur.
3. Bir kişi seçin ve ağacı büyütmek için **＋ Ebeveyn**, **＋ Partner**, **＋ Çocuk** veya **＋ Kardeş** (*＋ Parent*, *＋ Partner*, *＋ Child*, *＋ Sibling*) düğmelerini kullanın. Ad, tarih, fotoğraf ve notları sağ tarafta doldurun.
4. **Farklı kaydet…** seçeneğini seçin ve bir ad verin. Bundan sonra değişiklikler otomatik kaydedilir.
5. Haritayı keşfetmek için **Görüntüleme**'ye geçin: kaydırmak için sürükleyin, yakınlaştırmak için tekerleği çevirin, her şeyi görmek için **Sığdır** (*Fit to view*) düğmesine basın, ayrıntılar için bir karta tıklayın.

**Android'de:** bir `.famtree` dosyası açın ([Kurulum → Android](#android) bölümüne bakın), kaydırmak için sürükleyin, yakınlaştırmak için iki parmağınızla sıkıştırıp açın. *Fit to view* her şeyi tek seferde gösterir, büyüteç kişi aramasını açar, bir karta dokunmak kişi sayfasını açar (bir akrabaya dokunarak ona geçebilirsiniz). Sistemin Geri düğmesi veya hareketi sayfayı ya da aramayı adım adım kapatır; haritada ise uygulamayı arka plana gönderir. Güneş/ay düğmesi temayı değiştirir (varsayılan olarak uygulama sistem temasını izler).

## `.famtree` dosyası

Bir `.famtree` dosyası sıradan bir ZIP arşividir:

```
tree.json            people and relationships (plain, human-readable text)
media/<hash>.jpg     photos, resized to at most 1600 px
thumb/<hash>.jpg     small thumbnails for cards
```

- Fotoğraflar dosyanın içinde saklanır; bu yüzden tek bir `.famtree` dosyasını bir akrabaya gönderebilirsiniz ve hiçbir şey kaybolmaz. Aynı fotoğraflar yalnızca bir kez saklanır.
- `tree.json` herhangi bir metin düzenleyicisinde okunabilir ve her ilişkiyi açıklayan yorumlar içerir. Elle düzenlemek isterseniz arşivi açın, dosyayı düzenleyin ve yeniden paketleyin. Doğrudan `.famtree` içinde düzenlemek arşivi bozar.
- Eski dosyalar uygulamanın yeni sürümlerinde de açılmaya devam eder.

## Bilinen sınırlamalar

- Arayüz çevirileri (Rusça dışında) henüz ana dili konuşanlar tarafından gözden geçirilmedi. Düzeltmeler issue üzerinden memnuniyetle karşılanır.
- Tarihler dilden bağımsız olarak her zaman `GG.AA.YYYY` biçiminde gösterilir.
- Henüz geri alma yok.
- Çok sayıda fotoğrafı bir anda içe aktarmak pencereyi kısa süreliğine dondurabilir.
- HEIC fotoğraflar desteklenmez (JPEG/JFIF, PNG, WebP ve BMP desteklenir).
- Karttaki küçük resim ortadan kırpılmış bir görüntüdür ve kırpma alanını henüz seçemezsiniz.
- Çok büyük ailelerde yerleşim hâlâ kişileri tuhaf yerlere koyabilir veya dallar arasındaki çizgileri kesiştirebilir (yol haritasına bakın).
- Uyruklar Liste modunda gösterilmez.
- Android: yalnızca görüntüleme (düzenleme, Liste modu ve dışa aktarma yok), Android 10 veya üstü 64 bit ARM cihazlar, yalnızca APK (Google Play'de yok).
- Android: fotoğraflar henüz iki parmak hareketiyle büyütülemez ve dil menüsü yoktur: uygulama sistem dilini izler.

## Yol haritası

Proje tamamlandı: Windows uygulaması ve Android görüntüleyici hazır. Bundan sonra yalnızca küçük düzeltmeler planlanıyor; özellikle çok büyük ailelerde kişilerin garip yerlere yerleşmesi gibi nadir yerleşim hataları.

## Kaynaktan derleme

Yerel olarak derlemeyi tercih ederseniz, Windows'ta Rust araç zincirine ihtiyacınız var:

```
cargo test --release
cargo build --release
```

Sonuç `target/release/famtree.exe` olur. Proje arayüz için [egui/eframe](https://github.com/emilk/egui) kullanır.

Android uygulaması (`android/`) yalnızca `.github/workflows/android.yml` GitHub Actions iş akışıyla derlenir (cargo-apk, Android SDK/NDK, küçük bir Java katmanı ve imzalama). `src/` içindeki masaüstü kodunu (`model`, `container`, `layout`, `view`, `flags` ve diğerleri) derlediği için harita iki platformda da aynıdır. APK'yı kendi anahtarınızla imzalamak için `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS` ve farklıysa `ANDROID_KEY_PASSWORD` depo gizli anahtarlarını (secrets) ekleyin; eklemezseniz iş akışı geçici bir anahtar kullanır.

## Hata bildirimi

Bir sorun mu buldunuz, özellikle kişilerin haritada nasıl yerleştirildiğiyle ilgili? Lütfen [bir issue açın](https://github.com/john3sobieski/family-tree/issues) ve mümkünse sorunu yeniden üreten küçük bir `.famtree` dosyası ekleyin (ağaç özelse gerçek kişisel veriler olmadan). Android uygulamasında Android sürümünüzü ve cihaz modelinizi de belirtin.

## Lisans

FamTree, [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/) kapsamında lisanslanmıştır. [`LICENSE.md`](LICENSE.md) dosyasına bakın.

Kısaca:

- Uygulamayı ve kaynak kodunu **ticari olmayan** amaçlarla (kişisel kullanım, aile tarihi, hobi, eğitim, araştırma ve benzeri) kullanabilir, kopyalayabilir, değiştirebilir ve paylaşabilirsiniz.
- **Satmak veya ticari amaçla kullanmak yasaktır.**
- Uygulamayı veya değiştirilmiş bir sürümünü (fork, yeniden derleme, yeniden dağıtım) her paylaştığınızda, lisans koşullarını ve `LICENSE.md` içindeki, john3sobieski'yi yazar olarak belirten `Required Notice` satırını korumalısınız.

Bu özet hukuki tavsiye değildir; geçerli olan lisans metnidir.
