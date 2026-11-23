# Nahrok deneme sürümü: kurulum

Nahrok Microsoft Store'a çıkana kadar bu klasördeki deneme paketiyle kurulur. Windows 10 (2004 ya da daha yeni) veya
Windows 11 gerekir.

## Kurmak

1. Reponun GitHub ana sayfasında (github.com/nahrok-voice-app/nahrok-desktop) yeşil **Code** düğmesine bas,
   **Download ZIP**'i seç.
2. İnen ZIP dosyasına sağ tıkla, **Tümünü ayıkla**'yı seç.
3. Ayıklanan klasörde `release` klasörüne gir ve **Kur.cmd**'ye çift tıkla.
   - Windows "Windows bilgisayarınızı korudu" derse **Ek bilgi** > **Yine de çalıştır**'a bas.
   - Yönetici izni penceresinde **Evet**'e bas. Bu yalnız ilk kurulumda sorulur: Windows'a Nahrok'un deneme sertifikasını
     tanıtır.
4. "Nahrok kuruldu" yazısını görünce pencereyi kapat. Nahrok'u Başlat menüsünde "Nahrok" diye aratıp açabilirsin.

Yeni sürüm çıkınca aynı adımları tekrarla; eski sürümün üstüne kurulur, ayarların kalır. Deneme sürümü kendiliğinden
güncellenmez.

## Kaldırmak

`release` klasöründe **Kaldır.cmd**'ye çift tıkla. Nahrok'u ve deneme sertifikasını kaldırır (sertifika için yine yönetici
izni ister).

## Bilmen gerekenler

- **Smart App Control:** Windows Güvenliği'nde Akıllı Uygulama Denetimi açıksa Windows bu deneme paketini engelleyebilir.
  Store sürümünde bu sorun olmayacak.
- **Sertifika:** Kurulum, bu klasördeki `nahrok-test.cer` sertifikasını Windows'un güvenilir kişiler listesine ekler.
  Bilgisayarın bu sertifikayla imzalanmış paketlere güvenir; Kaldır.cmd sertifikayı da siler.
- **Ayarlar:** Tepsideki Nahrok simgesine sağ tıklayıp **Ayarlar**'ı seç (ya da Nahrok açıkken Ctrl + ,). Kısayollar,
  bilgisayar açılınca başlatma ve pencereyi kapatınca tepside çalışma oradadır.
