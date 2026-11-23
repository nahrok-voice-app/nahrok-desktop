# Kaldığım yer (2026-11-23)

Teknoloji **Tauri** (ölçümler PLAN.md ve deneme/KONTROL.md'de). Commit edildi, push edilmedi: nahrok-desktop `master`
(ilk commit, uzak repo yok) ve nahrok-client `masaustu-kopru` (develop'tan bir commit ileride).

## Hazır ve sınandı

- `uygulama/` derlendi (Smart App Control kapatıldı). `cargo test` 3/3, `npm run duman` 18/18.
- Sürüm derlemesi: `nahrok.exe` 5,4 MB; `NAHROK_URL`'yi yok sayıyor, ikilide localhost izni yok.
- `out/nahrok-1.0.0-yerel.msix` 2,4 MB, `CN=Nahrok Yerel` imzalı, Defender temiz.
- `nahrok-client` dalı `masaustu-kopru`: desktop.js, DesktopShortcuts.jsx, VoiceChannel/Chat eklemeleri,
  public/masaustu.json, test/desktop.test.mjs. Testler ve `vite build` geçiyor.
- Canlı site `masaustu.json` yerine HTML döndürüyor (dal yayında değil); uygulama bunu yok sayıyor, sürüm denetimi yapılmıyor.

## Sıra

1. Kullanıcı yönetici PowerShell'de `uygulama\yerel-kur.ps1` çalıştırır, sonra ELLE-SINAMA.md'yi gerçek sitede dener.
   Köprü özellikleri (rozet, kısayol, ses durumu) ancak `masaustu-kopru` dalı ekorhan.site'a çıkınca görünür.
2. Push kullanıcıya sorulur: nahrok-desktop için önce GitHub'da repo gerekir; `masaustu-kopru` develop'a birleşince
   site yayınına girer.
3. Store: Partner Center'dan `msix/magaza.json` doldurulur, `npm run paketle`, çoklu antivirüs taraması, gönderim.
