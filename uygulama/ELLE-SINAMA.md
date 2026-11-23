# Elle sınama listesi (her sürümden önce, Store'a göndermeden)

Otomatik duman testi (`npm run derle:sinama && npm run duman`) köprüyü, izinleri, gezinmeyi, kısayolu, tepsiyi, tek kopyayı,
sürüm ve çevrimdışı sayfalarını yerel sahte siteye karşı sınar. Aşağıdakiler gerçek sitede ve gerçek insanlarla yapılır.

Kurulum: `npm run paketle:yerel`, sonra yönetici PowerShell'de `.\yerel-kur.ps1`. Bitince `.\yerel-kur.ps1 -Kaldir`.

1. Başlat menüsünden **Nahrok** açılır, `https://ekorhan.site` gelir, giriş yapılır, sohbet yazılır.
2. Sesli kanala girilir: mikrofon izni sorulmadan çalışır (Windows gizlilik ayarında mikrofon açıksa), karşı taraf duyar.
3. Yayın: Ekran Paylaş > Paylaşmaya Başla > **Tüm Ekran** + **Sistem sesiyle paylaş**. İzleyen oyunun/videonun sesini duyar,
   **kanaldaki konuşmaları yayından ikinci kez duymaz**.
4. Başkasının yayını izlenir, tam ekran düğmesi pencereyi kaplar, Esc ile çıkılır.
5. Ses Ayarları > Kısayollar: varsayılan Ctrl+Shift+F9 / Ctrl+Shift+F10. Bir oyun ya da başka pencere öndeyken basınca
   mikrofon / ses kapanıp açılır.
6. Başka kanalda mesaj gelince görev çubuğu simgesinde sayı rozeti çıkar; kanal açılınca kalkar.
7. Seste iken pencere kapatılır: uygulama tepsiye iner, ses sürer; tepsi simgesine tıklayınca geri gelir. Tepsi > Çıkış kapatır.
   Seste değilken pencereyi kapatmak uygulamayı kapatır.
8. Sohbette bir bağlantıya tıklanır: sistem tarayıcısında açılır, uygulama yerinde kalır.
9. Uygulama açıkken ikinci kez açılır: yeni pencere açılmaz, var olan öne gelir.
10. İnternet kesilip uygulama açılır: "Nahrok'a bağlanılamadı" sayfası; bağlantı gelince kendiliğinden siteye döner.
