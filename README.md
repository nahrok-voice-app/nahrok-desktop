# Nahrok masaüstü

Nahrok'u (`https://ekorhan.site`) Windows'ta ayrı bir uygulama olarak açan küçük bir Tauri kabuğu. Arayüz siteden gelir;
uygulama yalnız masaüstüne özgü işleri yapar: oyundayken çalışan mikrofon / ses kısayolları, tepsi, okunmamış rozeti,
bilgisayar açılınca başlatma ve sistem sesiyle yayın. Ne yapılıp ne yapılmayacağı ve nedenleri: [PLAN.md](PLAN.md).

**Arkadaşına kurmak için:** [release/](release/) klasöründeki adımlar.

## Yerleşim

| Yol | İçerik |
|---|---|
| `src-tauri/src/` | Rust kodu. `main.rs` modüllerin ne yaptığını listeler. |
| `src-tauri/src/bridge.js` | Sitenin gördüğü `window.nahrokDesktop` köprüsü |
| `src-tauri/capabilities/` | Hangi komutu kimin çağırabileceği (site, ayarlar penceresi, duman testi) |
| `ui/` | Uygulamanın kendi sayfaları: ayarlar, "bağlanılamadı", "güncelleme gerekli". Görsel dil `nahrok-client/TASARIM.md`. |
| `msix/` | MSIX paket bildirimi ve Store kimliği (`store.json`, Partner Center'dan doldurulur) |
| `scripts/package.ps1` | MSIX paketi üretir, Defender'la tarar |
| `tests/smoke.js` | Duman testi (yerel sahte siteye karşı) |
| `release/` | Arkadaşlar için deneme paketi ve kurulum (Store'a çıkana kadar) |
| `docs/` | Teknoloji denemesi sonuçları, elle sınama listesi |
| `assets/icon.svg` | Uygulama simgesi (sitenin simgesi; `npx tauri icon assets/icon.png -o src-tauri/icons` ile boyutlanır) |

Kodda adlar İngilizce, açıklamalar ve arayüz metinleri Türkçedir (`nahrok-client` ile aynı).

## Geliştirme

Gerekenler: Rust (stable, MSVC), Node.js, Visual Studio Build Tools ve Windows SDK (paketleme için).

```powershell
npm install
npm test                  # Rust birim testleri
npm run build:test        # duman testi için hata ayıklama derlemesi (localhost'a izin verir)
npm run test:smoke        # duman testi: pencereler birkaç kez açılıp kapanır
npm run package:test      # deneme paketi: release\ klasörüne imzalı MSIX + sertifika
npm run package           # Store paketi: out\ (önce msix\store.json doldurulur)
```

Hata ayıklama derlemesi `NAHROK_URL` ortam değişkeniyle başka bir adresi açabilir; yayın derlemesinde bu yoktur.

Yeni deneme sürümü çıkarırken `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` ve `package.json` içindeki sürüm
birlikte artırılır; Windows aynı sürümü yeniden kurmaz.
