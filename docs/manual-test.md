# Elle sınama listesi

Her sürümden önce, Store'a göndermeden yapılır. Otomatik duman testi (`npm run build:test && npm run test:smoke`) köprüyü,
izinleri, gezinmeyi, kısayolu, tepsiyi, tek kopyayı, ayarlar penceresini, sürüm ve "bağlanılamadı" sayfalarını yerel
sahte siteye karşı sınar. Aşağıdakiler gerçek sitede ve gerçek insanlarla yapılır.

Kurulum: `npm run package:test`, sonra `release\Kur.cmd`. Kaldırmak için `release\Kaldır.cmd`.

1. Başlat menüsünden **Nahrok** açılır, `https://ekorhan.site` gelir, giriş yapılır, sohbete yazılır.
2. Sesli kanala girilir: mikrofon izni sorulmadan çalışır (Windows gizlilik ayarlarında mikrofon açıksa), karşı taraf duyar.
3. Yayın: Ekran Paylaş > Paylaşmaya Başla > **Tüm Ekran** + **Sistem sesiyle paylaş**. İzleyen oyunun ya da videonun sesini
   duyar, **kanaldaki konuşmaları yayından ikinci kez duymaz**.
4. Başkasının yayını izlenir; tam ekran düğmesi pencereyi kaplar, Esc ile çıkılır.
5. Kısayollar: varsayılan Ctrl+Shift+F9 (mikrofon) ve Ctrl+Shift+F10 (ses). Bir oyun ya da başka pencere öndeyken basınca
   mikrofon / ses kapanıp açılır. Ayarlardan başka bir tuşa alınınca yenisi çalışır, eskisi çalışmaz.
6. Başka kanala mesaj gelince görev çubuğu simgesinde sayı rozeti çıkar; kanal açılınca kalkar.
7. Seste iken pencere kapatılır: uygulama tepsiye iner, konuşma sürer. Tepsi menüsünden "Mikrofonu kapat" / "Sesi kapat"
   çalışır; tepsi simgesine tıklayınca pencere geri gelir; "Çıkış" uygulamayı kapatır. Seste değilken pencereyi kapatmak
   uygulamayı kapatır (ayarlarda "Kapatınca tepside çalışsın" açıksa tepsiye iner).
8. Ayarlar üç yoldan açılır: tepsi menüsü > Ayarlar, ana pencerede Ctrl + , ve sitede Ses Ayarları > "Uygulama ayarlarını aç".
9. **Bilgisayar açılınca başlat** açılır, bilgisayar yeniden başlatılır: Nahrok pencere açmadan tepside başlar ("Tepside
   başlat" kapalıysa pencereyle açılır). Ayar kapatılınca bir sonraki açılışta başlamaz. Windows Ayarlar > Uygulamalar >
   Başlangıç'ta kapatılırsa ayarlar penceresi bunu söyler ve "Windows ayarlarını aç" düğmesi oraya götürür.
10. Sohbette bir bağlantıya tıklanır: sistem tarayıcısında açılır, uygulama yerinde kalır.
11. Uygulama açıkken ikinci kez açılır: yeni pencere açılmaz, var olan öne gelir.
12. Pencere büyütülüp taşınır, uygulama kapatılıp açılır: aynı boyut ve yerde açılır.
13. İnternet kesilip uygulama açılır: "Nahrok'a bağlanılamadı" sayfası çıkar; bağlantı gelince kendiliğinden siteye döner.
