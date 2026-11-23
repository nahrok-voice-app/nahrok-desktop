# Deneme: kontrol listesi ve sonuçlar (PLAN.md, Sıra 1)

Deneme 2026-11-21/22'de repo sahibinin Windows 11 (26300) bilgisayarında yapıldı. Sonuç: **Tauri**.

| # | Soru | Electron 39 | Tauri 2.12 / WebView2 157 |
|---|---|---|---|
| 1 | Site açılıyor | evet (boş kabukta) | evet |
| 2 | Mikrofon izni | handler ile | WebView2 izin olayı ile (v1'de yalnız mikrofon + otomatik oynatma) |
| 3 | Başkasının yayını izlenir | (gerçek kanalda sınanmadı; aynı Chromium) | (gerçek kanalda sınanmadı; aynı Chromium) |
| 4 | Seçici çıkıyor, seçim olmadan yakalama yok | kendi menümüz | WebView2'nin kendi seçicisi: "Pencere / Tüm Ekran", "Sistem sesiyle paylaş" |
| 5 | Sistem sesiyle yayın | **evet** (dış ses -40 dB) | **evet** (dış ses -40 dB) |
| 6 | Kanal konuşmaları yayına karışmıyor (`restrictOwnAudio`) | **hayır**: kendi ses -35 dB (`loopback`, `loopbackWithoutChrome`, `RestrictOwnAudio` bayrağı denendi) | **evet**: kendi ses -89 dB (`loopbackWithoutChrome`, taban -165 dB) |
| 7 | Dış bağlantı sistem tarayıcısında | `shell.openExternal` | `opener` (v1, duman testinde) |
| 8 | MSIX üretilir / kurulur | 135 MB üretildi (önbellek `%LOCALAPPDATA%` dışına alınınca); kurulum için sertifika güveni gerekir | 2,5 MB üretildi (`uygulama/paketle.ps1`), Defender temiz; kurulum `yerel-kur.ps1` (yönetici) |

Ölçüm düzeneği `electron/sinama/`: `ses-sinama.html` uygulama içinde 440 Hz çalar, ayrı bir PowerShell işlemi 1000 Hz çalar,
`getDisplayMedia` ses izinde iki frekansın gücüne bakılır. Electron: `npx electron sinama/ses-sinama.js loopback`.
Tauri: `node sinama/webview2-ses-sinama.js <exe>` + `powershell -File sinama/webview2-secici-sur.ps1` (seçiciyi UI
Automation ile geçer; yalnız 'Nahrok' penceresine bakar).

Bulunan engeller:
- electron-builder önbelleği `%LOCALAPPDATA%` altındayken `makeappx.exe` "yan yana yapılandırma" hatası veriyor.
- İmzasız MSIX tam güven uygulamasını Windows kurmuyor ("unsigned package cannot include Executable activations").
- Smart App Control açık: yerel Rust derlemesinin derleme betikleri engelleniyor (PLAN.md tespitler). SAC kapatılınca
  `uygulama/` derlendi; `npm run duman` 18/18 (2026-11-23).
- Chrome karşılaştırması yapılamadı: otomatik seçim bayraklarıyla Chrome sessiz ses izi veriyor.
