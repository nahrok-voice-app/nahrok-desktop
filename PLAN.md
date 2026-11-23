# Nahrok masaüstü sürümü: plan ve sınırlar

Son güncelleme: 2026-11-22 (dördüncü sürüm: deneme bitti, teknoloji Tauri; site adresi ve köprü listesi yazıldı).
Durum: **v1 derlendi, otomatik duman testi 18/18**, yerel MSIX hazır. Kalan: elle sınama (ELLE-SINAMA.md), web dalının
yayını, Store gönderimi.

Amaç: arkadaşların bilgisayarlarına güvenle kurabileceği, virüs uyarısı vermeyen bir Nahrok masaüstü uygulaması. Bu dosya ne
yapılacağını ve neyin yapılmayacağını söyler; uygulama ayrıntısı içermez.

## Özet

Arayüzü canlı siteden (`https://ekorhan.site`) yükleyen küçük bir Windows uygulaması. Ayrı bir repoda (`nahrok-desktop`)
yaşar ve yalnız Microsoft Store'dan, MSIX paketi olarak dağıtılır. Teknoloji **Tauri** (deneme sonucu, aşağıda); kod
`uygulama/`, deneme kayıtları `deneme/`.

## Planı belirleyen tespitler

- **Güncel dal:** Güncel kod `develop` dallarında, `main`/`master` Mart 2026'da kalmış. Masaüstü için istemcide yapılacak her iş
  `develop`'tan dallanmalı.
- **Adresler:** İstemci sunucuya hep kendi açıldığı adres üzerinden gidiyor (`src/api/axios.js`, `src/store/useVoiceStore.js`
  içindeki `io(...)`, `src/pages/Chat.jsx` içindeki `brokerURL`). Site uygulamanın içinde açılırsa bu üç yer değişmeden çalışır;
  arka uçta da değişiklik gerekmez.
- **Fikir atölyesi:** `nahrok-fikir-atolyesi` istemciyi insan görmeden değiştirip canlıya çıkarıyor ve arka ucu istemciden önce
  güncelliyor. Yani web arayüzü sık ve denetimsiz değişiyor.
- **Tarayıcı:** Ses ve yayın kodu Chromium'a göre ayarlı, `nahrok-e2e` testleri de yalnız Chromium'da koşuyor.
- **Geliştirme makinesi:** Smart App Control açık. Yerelde derlenen imzasız programları (Cargo'nun derleme betikleri dahil)
  çoğu zaman engelliyor; Rust derlemesi bu makinede ancak SAC kapalıyken ya da başka bir makinede (ör. CI) yapılabilir.
  2026-11-23'te repo sahibi SAC'yi kapattı.
  Store'dan kurulan, Microsoft imzalı paket bundan etkilenmez.

## Kesin çizgiler

Bu çizgilerden biri değişecekse önce bu dosya değişir, sonra kod.

1. **Arayüz tek yerde yaşar.** Uygulama siteyi yükler, içine ayrı bir arayüz kopyası gömülmez. Gömülü kopya, atölye arka ucu her
   değiştirdiğinde geride kalıp kırılır.
2. **Site, uygulamanın gözünde güvenilmezdir.** Web içeriği işletim sistemine ve dosyalara doğrudan erişemez (Electron'da
   `sandbox` ve `contextIsolation` açık, `nodeIntegration` kapalı; Tauri'de yalnız izin listesindeki komutlar). Yalnız tek sabit
   adres yüklenir; başka adrese gezinme ve yeni pencere engellenir, dış bağlantılar sistem tarayıcısında açılır. İzin olarak
   yalnız mikrofon, ekran yakalama ve tam ekran verilir (bir de sesli kanal sesinin çalabilmesi için otomatik oynatma; kamera,
   konum, bildirim, pano okuma ve diğerleri reddedilir). Sertifika hatası hiçbir koşulda atlanmaz.
3. **Web'in uygulamadan isteyebilecekleri kısa bir listedir.** Bugünkü liste: `rozet(sayı)`, `sesDurumu({kanalda,
   susturuldu, sagir})`, `kisayollar(değişiklik?)`; uygulamadan web'e yalnız `kisayolDinle` ile 'sustur' / 'sagirlastir'.
   Liste dışı her çağrı reddedilir (`uygulama/src-tauri/build.rs` + `capabilities/site.json`, komutta ayrıca köken
   denetimi); listeye ekleme bu dosyada yazılı kapsamı değiştirmek demektir.
4. **Paylaşılacak ekranı kullanıcı seçer.** Seçim uygulamanın ya da tarayıcı çekirdeğinin kendi penceresinde yapılır. Web kodu
   kaynak seçemez, seçim olmadan yakalama başlamaz.
5. **Web bozulmaz.** Web kodu masaüstü uygulamasını algılarsa kullanır, yoksa bugünkü gibi çalışır. TV ve telefon web'de kalır.
6. **Atölye masaüstüne dokunmaz.** Uygulama ayrı repodadır. Sürümü insan çıkarır; Store hesabı ve yükleme yetkisi ajan
   konteynerlerinin erişemeyeceği yerde durur.
7. **Tek dağıtım yolu Microsoft Store'dur.** Paket MSIX'tir; elden exe dağıtılmaz. Kurulum yönetici izni istemez, sürücü ya da
   arka plan servisi kurmaz, otomatik başlatma varsayılan kapalıdır.
8. **Tarayıcı çekirdeği güncel kalır.** Tauri'de çekirdek Windows'un güncellediği WebView2'dir, sabit sürüm gömülmez. Electron'da
   desteği bitmiş sürümle yayın yapılmaz (yaklaşık iki ayda bir ana sürüm çıkar, son üçü yamalanır).
9. **Antivirüsün işaretlediği paket yüklenmez.** Her sürüm Store'a gönderilmeden önce çoklu antivirüs taramasından geçer;
   Microsoft Defender ya da yaygın bir antivirüs işaretliyorsa gönderim durur, yanlış alarm bildirimi yapılır.

## Kararlar

### Platform (kapandı)

Yalnız Windows. Arkadaşlar arasında Mac kullanan yok; macOS ve Linux kapsam dışı, o cihazlar web'i kullanır.

### Dağıtım (kapandı)

Microsoft Store, MSIX paketi olarak.

- **Neden Store:** Kayıt ücretsiz, paketi Microsoft imzalar, kurulumda uyarı çıkmaz, güncellemeler arkadaşlara kendiliğinden
  ulaşır.
- **Neden MSIX şart:** Store yalnız MSIX paketlerini ücretsiz imzalar. Exe ya da msi olarak yüklenen uygulamayı imzalamaz;
  onlar için kendi ücretli sertifikamız gerekirdi.
- **Store'un istedikleri:** Kimlik doğrulamalı geliştirici hesabı (repo sahibi açar), gizlilik sayfası, inceleyici için bir
  test hesabı. Bağlantıyla gizli listeleme seçeneği ve inceleme süresi başvuruda doğrulanacak.
- **Elenenler:** Sertifikayla imzalı doğrudan exe (yılda kabaca 120–220 dolar, ilk dönemde SmartScreen uyarısı çıkabilir) ve
  imzasız exe (her sürümde "bilinmeyen yayımcı" uyarısı; Smart App Control açık bilgisayarda hiç çalışmaz).

### Teknoloji (kapandı: Tauri)

Deneme 2026-11-21/22'de bu makinede yapıldı (`deneme/KONTROL.md`). Belirleyen ölçüm: uygulama içinde 440 Hz, ayrı bir
işlemde 1000 Hz çalarken "sistem sesiyle tüm ekran" yayınının sesi.

| | Dış ses (1000 Hz) | Kendi ses (440 Hz), `restrictOwnAudio: true` |
|---|---|---|
| Electron 39 (`loopback`, `loopbackWithoutChrome`, `RestrictOwnAudio` bayrağı) | -40 dB | -35 dB: **konuşmalar yayına karışıyor** |
| Tauri / WebView2 157 (kendi seçicisi, "Sistem sesiyle paylaş") | -40 dB | **-89 dB**: ayrılıyor (taban -165 dB) |

MSIX: Tauri exe'si Windows SDK `makeappx` ile 2,5 MB pakete dönüşüyor (`uygulama/paketle.ps1`); Electron'unki 135 MB.
İki koşul da tuttuğu için Tauri seçildi. Aşağıdaki tablo seçim öncesi değerlendirmedir.

| Seçenek | Artı | Eksi |
|---|---|---|
| **Tauri (deneme geçerse)** | Tarayıcı çekirdeğini (WebView2) Windows kendisi günceller, iki ayda bir güncelleme işi yok; paket küçük; ekran seçici hazır gelir | Sistem sesiyle yayının WebView2'de çalıştığı doğrulanmadı; MSIX paketi ek bir araç ister; otomatik test mevcut Linux test ortamında koşamaz; uygulama kodu Rust |
| Electron | Chromium gömülü: testlerdeki tarayıcının aynısı, yakalama tam kontrolde; MSIX paketi hazır araçla üretilir; duman testi mevcut test ortamında koşar | Yaklaşık 100 MB; iki ayda bir güncelleme şart; ekran seçiciyi biz yazarız |

Tauri'nin seçilmesi için iki koşul birden gerekir: sistem sesiyle yayın çalışmalı ve MSIX paketi üretilip kurulabilmeli. Biri
tutmazsa Electron seçilir. (Sonuç: ikisi de tuttu; kurulum adımı yönetici izni istediği için repo sahibi `yerel-kur.ps1` ile sınar.)

### Arayüz nereden gelir (kapandı)

Siteden yüklenir (çizgi 1). Bedeli: açılış internet ister; site ele geçirilirse arayüz de etkilenir, çizgi 2 zararı web düzeyinde
tutar.

### Güncelleme sorumluluğu (kapandı)

Sorumlu, sürüm çıkarabilen tek insan olan repo sahibidir; atölye üstlenemez (çizgi 6). Yeni sürümler arkadaşlara Store'dan
kendiliğinden ulaşır. İşin büyüklüğü teknolojiye bağlı:

- **Tauri seçilirse:** Tarayıcı çekirdeğini Windows günceller. Geriye yılda birkaç kez uygulamanın kendi bağımlılıklarını
  güncellemek kalır.
- **Electron seçilirse:** İki ayda bir güncelleme şart. Güncelleme isteğini bağımlılık botu açar, duman testi otomatik koşar;
  insana elle bir ses ve yayın denemesi ile sürümü Store'a göndermek kalır.

## v1 kapsamı

**Var**

- Giriş, sohbet, ses, yayın izleme
- Yayın açma (kaynak seçimi çizgi 4'e göre)
- Oyundayken çalışan sustur/sağırlaştır kısayolu (bas-değiştir)
- Tepsi ikonu ve pencere kapanınca seste kalma
- Okunmamış rozeti
- Tek kopya çalışma
- En düşük sürüm kontrolü (eski uygulama "Store'dan güncelle" ekranı görür)

**Yok**

- Bas-konuş: tuşun bırakılmasını yakalamak, antivirüslerin tuş kaydedici saydığı türden bir klavye kancası ister
- Oyun içi katman
- Uygulama bazında ses yakalama
- Çevrimdışı kullanım
- Sunucu adresi seçme
- Uygulama içi güncelleyici (güncellemeyi Store yapar)
- macOS ve Linux sürümü

## Sıra

0. **Store hesabı (repo sahibi, hemen).** Kimlik doğrulaması zaman alabildiği için denemeyle aynı anda başlar.
   Hesap açılınca Ürün kimliği değerleri `uygulama/msix/magaza.json`'a yazılır.
1. **Deneme (bitti: Tauri).** İki boş uygulamada (Tauri ve Electron) site açılıp bir Windows bilgisayarda sınanır (iskeletler `deneme/`
   klasöründe, adımlar `deneme/KONTROL.md` içinde):
   - Giriş, ses ve yayın çalışıyor mu?
   - Sistem sesiyle yayın açılabiliyor mu, kanaldaki konuşmalar yayına geri karışıyor mu? Web'de bunu `getDisplayMedia`
     çağrısındaki `restrictOwnAudio` çözüyor (`src/components/VoiceChannel.jsx`).
   - MSIX paketi üretilip yerelde kurulabiliyor mu?

   Sonuç teknolojiyi belirler. İkisinde de sistem sesi çalışmazsa v1'de yayın sesi varsayılan kapalı gelir.
2. **Uygulama (yazıldı: `uygulama/`).** Çizgi 2–4, tepsi ve kısayol. Windows'ta otomatik duman testi
   (`npm run derle:sinama && npm run duman`, yerel sahte siteye karşı) ve `uygulama/ELLE-SINAMA.md`.
3. **Web tarafı (yazıldı: `nahrok-client` `masaustu-kopru` dalı, `develop`'tan).** `src/utils/desktop.js`, rozet,
   kısayol, ses durumu, masaüstü yayın metni, `public/masaustu.json` (en düşük sürüm).
4. **Yayın.** Gizlilik sayfası ve inceleyici test hesabı, çizgi 9'daki tarama, Store'a gönderim; 2–3 arkadaşla deneme, sonra
   herkese.

## Açık kararlar

Yok. Teknoloji seçimi karar değil, 1. adımdaki denemenin sonucudur.

## Kaynaklar

- [Microsoft Store: bireysel geliştiricilere ücretsiz kayıt](https://blogs.windows.com/windowsdeveloper/2025/09/10/free-developer-registration-for-individual-developers-on-microsoft-store/)
- [Microsoft Store SSS: MSIX paketini Microsoft imzalar](https://learn.microsoft.com/en-us/windows/apps/publish/faq/get-started-with-the-microsoft-store)
- [Microsoft: Win32 uygulamasını Store'dan dağıtma (exe/msi imza şartı)](https://learn.microsoft.com/en-us/windows/apps/distribute-through-store/how-to-distribute-your-win32-app-through-microsoft-store)
- [Microsoft: Tauri uygulamasını MSIX olarak paketleme](https://learn.microsoft.com/nb-no/windows/apps/dev-tools/winapp-cli/guides/tauri)
- [Tauri: Windows kod imzalama](https://v2.tauri.app/distribute/sign/windows/)
- [Microsoft: kod imzalama seçenekleri ve SmartScreen](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)
- [Smart App Control mevcut kurulumlarda açılabiliyor (Nisan 2026)](https://blog.ciaops.com/2026/04/16/existing-systems-can-now-enable-windows-smart-app-control-and-you-should/)
- [Kod imzalama sertifikası fiyatları (2026)](https://codenote.net/en/posts/ov-ev-code-signing-certificate-pricing-japan-2026/)
- [Chromium: restrictOwnAudio](https://groups.google.com/a/chromium.org/g/blink-dev/c/5SZfg29iwPc)
- [Electron: desktopCapturer ve ekran/ses yakalama](https://www.electronjs.org/docs/latest/api/desktop-capturer)
