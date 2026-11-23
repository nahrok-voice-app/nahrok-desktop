# Teknoloji denemesi: Tauri mi Electron mu?

Deneme 2026-11-21/22'de repo sahibinin Windows 11 (26300) bilgisayarında yapıldı. Sonuç: **Tauri** (PLAN.md, "Teknoloji").
Deneme kodu (iki boş uygulama ve ölçüm düzeneği) ilk commit'te durur: `git show bebd6d2 --stat -- deneme/`.

| # | Soru | Electron 39 | Tauri 2.12 / WebView2 157 |
|---|---|---|---|
| 1 | Site açılıyor mu? | Evet (boş kabukta) | Evet |
| 2 | Mikrofon izni | İzin işleyicisiyle | WebView2 izin olayıyla (v1'de yalnız mikrofon ve otomatik oynatma) |
| 3 | Başkasının yayını izlenebiliyor mu? | Gerçek kanalda sınanmadı (aynı Chromium) | Gerçek kanalda sınanmadı (aynı Chromium) |
| 4 | Seçici çıkıyor mu, seçim olmadan yakalama başlıyor mu? | Seçiciyi biz yazardık | WebView2'nin kendi seçicisi: "Pencere / Tüm Ekran" ve "Sistem sesiyle paylaş"; seçimsiz yakalama yok |
| 5 | Sistem sesiyle yayın | **Evet** (dış ses -40 dB) | **Evet** (dış ses -40 dB) |
| 6 | Kanaldaki konuşmalar yayına karışmıyor mu? (`restrictOwnAudio`) | **Karışıyor**: kendi ses -35 dB (`loopback`, `loopbackWithoutChrome` ve `RestrictOwnAudio` bayrağı denendi) | **Karışmıyor**: kendi ses -89 dB (`loopbackWithoutChrome`, taban -165 dB) |
| 7 | Dış bağlantı sistem tarayıcısında açılıyor mu? | `shell.openExternal` | `opener` eklentisi (duman testinde) |
| 8 | MSIX üretilip kurulabiliyor mu? | 135 MB üretildi; kurulum için sertifika güveni gerekir | 2,5 MB üretildi, Defender temiz, kuruldu |

## Ölçüm yöntemi

Uygulamanın içindeki sayfa 440 Hz, ayrı bir PowerShell işlemi 1000 Hz ton çalar. "Sistem sesiyle tüm ekran" yayını
başlatılır ve yayının ses izinde iki frekansın gücüne bakılır: 1000 Hz duyuluyor ve 440 Hz duyulmuyorsa uygulama kendi
sesini (kanaldaki konuşmaları) yayından ayırıyor demektir. WebView2 seçicisi UI Automation ile geçildi; betik yalnız Nahrok
penceresine bakar.

## Yolda çıkan engeller

- electron-builder önbelleği `%LOCALAPPDATA%` altındayken `makeappx.exe` "yan yana yapılandırma" hatası veriyor; önbellek
  başka klasöre alınınca çalışıyor.
- Windows imzasız MSIX içindeki tam güven uygulamasını kurmuyor ("unsigned package cannot include Executable
  activations"). Deneme paketi bu yüzden kendinden imzalı sertifikayla imzalanır; Store paketini Microsoft imzalar.
- Smart App Control açıkken yerel Rust derlemesinin derleme betikleri engelleniyor. Repo sahibi 2026-11-23'te kapattı.
- Chrome ile karşılaştırma yapılamadı: otomatik seçim bayraklarıyla Chrome sessiz ses izi veriyor.
