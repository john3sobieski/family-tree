[English](README.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | [Português (BR)](README.pt-BR.md) | [日本語](README.ja.md) | [Русский](README.ru.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [한국어](README.ko.md) | **Türkçe** | [Bahasa Indonesia](README.id.md)

# FamTree

**[john3sobieski](https://github.com/john3sobieski)** tarafından oluşturuldu ve sürdürülüyor. Depo: <https://github.com/john3sobieski/family-tree>

Aile ağacınızı oluşturmak, düzenlemek ve incelemek için Windows masaüstü uygulaması. Her şey tek bir `.famtree` dosyasında durur: kişiler, ilişkiler ve tüm fotoğraflar.

> **Durum:** erken geliştirme aşamasında. Henüz hazır sürüm yok ve uygulama şu anda yalnızca GitHub Actions ile derlenerek edinilebiliyor ([Kurulum](#kurulum) bölümüne bakın).

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
- **Çift tıklayarak açma.** **Dosya** (*File*) menüsünden `.famtree` dosyalarını uygulamayla ilişkilendirebilirsiniz.

## Kurulum

Uygulama Windows 10 ve Windows 11'de (aynı exe) çalışır ve GitHub Actions tarafından kaynak koddan derlenir; bu yüzden makinenizde Rust veya başka bir araca ihtiyacınız yoktur. Ancak bir GitHub hesabına ihtiyacınız var.

1. [Depoyu](https://github.com/john3sobieski/family-tree) **Fork** edin (veya kendi kopyanızı kullanın).
2. Fork'unuzda **Actions** sekmesini açın. GitHub isterse, fork için workflow'ları etkinleştirin.
3. **Build** workflow'unu seçin, **Run workflow**'a tıklayın ve bitmesini bekleyin (yaklaşık 2–3 dakika). Her push'ta da çalışır.
4. Tamamlanan çalıştırmayı açın ve en alttaki *Artifacts* bölümünden **`famtree-windows`** çıktısını indirin.
5. Arşivi açın. İçinde `famtree.exe` var. Kurulacak bir şey yok, doğrudan çalıştırın.

Notlar:

- `.exe` imzalı olmadığı için Windows bir SmartScreen uyarısı gösterebilir. Kendi derlemenize güveniyorsanız *Daha fazla bilgi → Yine de çalıştır* seçeneğini seçin.
- Artifact'ler GitHub tarafından sınırlı bir süre saklanır (varsayılan olarak 90 gün), bu yüzden `.exe` dosyasını kalıcı bir yere kaydedin.
- `famtree.exe` dosyasını taşırsanız, ilişkilendirme exe'nin konumunu gösterdiği için **Dosya** menüsündeki `.famtree` ilişkilendirme düğmesine yeniden basın.

Gerçek bir yükleyici hazır olduğunda **Releases** bölümünde yayımlanacak ve bu bölüm değiştirilecek.

## Hızlı başlangıç

1. `famtree.exe` dosyasını başlatın. Örnek bir ağaca bakmak için **Dosya → Demo** (*File → Demo*), kendi ağacınızı başlatmak için **Dosya → Yeni** (*File → New*) seçeneğini kullanın.
2. **Düzenleme** moduna geçin ve ilk kişiyi ekleyin (*＋ New person*). Boş bir ağaçta bu kişi otomatik olarak "ben" olur.
3. Bir kişi seçin ve ağacı büyütmek için **＋ Ebeveyn**, **＋ Partner**, **＋ Çocuk** veya **＋ Kardeş** (*＋ Parent*, *＋ Partner*, *＋ Child*, *＋ Sibling*) düğmelerini kullanın. Ad, tarih, fotoğraf ve notları sağ tarafta doldurun.
4. **Farklı kaydet…** seçeneğini seçin ve bir ad verin. Bundan sonra değişiklikler otomatik kaydedilir.
5. Haritayı keşfetmek için **Görüntüleme**'ye geçin: kaydırmak için sürükleyin, yakınlaştırmak için tekerleği çevirin, her şeyi görmek için **Sığdır** (*Fit to view*) düğmesine basın, ayrıntılar için bir karta tıklayın.

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
- Henüz geri alma ve PNG/PDF dışa aktarma yok.
- Çok sayıda fotoğrafı bir anda içe aktarmak pencereyi kısa süreliğine dondurabilir.
- HEIC fotoğraflar desteklenmez (JPEG, PNG, WebP ve BMP desteklenir).
- Karttaki küçük resim ortadan kırpılmış bir görüntüdür ve kırpma alanını henüz seçemezsiniz.
- Çok büyük ailelerde yerleşim hâlâ kişileri tuhaf yerlere koyabilir veya dallar arasındaki çizgileri kesiştirebilir (yol haritasına bakın).
- Uyruklar Liste modunda gösterilmez.

## Yol haritası

Planlanan iyileştirmeler, belirli bir sırayla değil:

- [ ] Actions'tan alınan çıplak bir `.exe` yerine Releases'ta yayımlanan **gerçek bir yükleyici**.
- [ ] **Yerleşim hatalarının düzeltilmesi**: kişiler zaman zaman haritada yanlış yere düşüyor.

## Kaynaktan derleme

Yerel olarak derlemeyi tercih ederseniz, Windows'ta Rust araç zincirine ihtiyacınız var:

```
cargo test --release
cargo build --release
```

Sonuç `target/release/famtree.exe` olur. Proje arayüz için [egui/eframe](https://github.com/emilk/egui) kullanır.

## Hata bildirimi

Bir sorun mu buldunuz, özellikle kişilerin haritada nasıl yerleştirildiğiyle ilgili? Lütfen [bir issue açın](https://github.com/john3sobieski/family-tree/issues) ve mümkünse sorunu yeniden üreten küçük bir `.famtree` dosyası ekleyin (ağaç özelse gerçek kişisel veriler olmadan).

## Lisans

FamTree, [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/) kapsamında lisanslanmıştır. [`LICENSE.md`](LICENSE.md) dosyasına bakın.

Kısaca:

- Uygulamayı ve kaynak kodunu **ticari olmayan** amaçlarla (kişisel kullanım, aile tarihi, hobi, eğitim, araştırma ve benzeri) kullanabilir, kopyalayabilir, değiştirebilir ve paylaşabilirsiniz.
- **Satmak veya ticari amaçla kullanmak yasaktır.**
- Uygulamayı veya değiştirilmiş bir sürümünü (fork, yeniden derleme, yeniden dağıtım) her paylaştığınızda, lisans koşullarını ve `LICENSE.md` içindeki, john3sobieski'yi yazar olarak belirten `Required Notice` satırını korumalısınız.

Bu özet hukuki tavsiye değildir; geçerli olan lisans metnidir.
