// Nahrok masaüstü. Kurallar ../../PLAN.md "Kesin çizgiler"dedir; bu dosya onları uygular.
//  - Çizgi 1: arayüz siteden yüklenir, içeride kopya yok.
//  - Çizgi 2: site güvenilmez. Tek adres; başka adrese gezinme ve yeni pencere yok, dış bağlantı sistem tarayıcısında;
//    izin yalnız mikrofon (+ sesli kanal için otomatik oynatma); ekran yakalama WebView2'nin kendi seçicisiyle (çizgi 4);
//    sertifika hatası atlanmaz (WebView2 varsayılanı, burada değiştirilmez).
//  - Çizgi 3: siteye açık komutlar yalnız rozet, ses_durumu, kisayollar (build.rs + capabilities/site.json).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, str::FromStr, sync::Mutex, time::Duration};

use serde::{Deserialize, Serialize};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::{NewWindowResponse, PermissionKind, PermissionResponse},
    AppHandle, Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_opener::OpenerExt;

const SITE: &str = "https://ekorhan.site";
const ANA: &str = "ana";
const TEPSI: &str = "tepsi";
const SURUM: &str = env!("CARGO_PKG_VERSION");
const STORE_VARSAYILAN: &str = "ms-windows-store://downloadsandupdates";

/// Yüklenen tek adres. Hata ayıklama derlemesinde duman testi için NAHROK_URL ile değiştirilebilir; yayında değiştirilemez.
fn site() -> Url {
    #[cfg(debug_assertions)]
    if let Some(u) = std::env::var("NAHROK_URL").ok().and_then(|u| u.parse().ok()) {
        return u;
    }
    SITE.parse().expect("SITE geçerli bir adres")
}

fn ayni_site(u: &Url) -> bool {
    u.origin() == site().origin()
}

/// Uygulamanın kendi yerel sayfaları (ui/): Windows'ta http://tauri.localhost
fn yerel_sayfa(u: &Url) -> bool {
    u.host_str() == Some("tauri.localhost") && matches!(u.scheme(), "http" | "https")
}

fn yerel(sayfa: &str) -> Url {
    Url::parse("http://tauri.localhost/").unwrap().join(sayfa).unwrap()
}

// ---------------------------------------------------------------- durum ve ayarlar

#[derive(Default)]
struct Durum {
    kanalda: bool,
    susturuldu: bool,
    sagir: bool,
    okunmamis: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct Kisayollar {
    sustur: String,
    sagirlastir: String,
}

impl Default for Kisayollar {
    fn default() -> Self {
        // Oyunlarda ve tarayıcılarda nadiren kullanılan birleşimler
        Self { sustur: "Ctrl+Shift+F9".into(), sagirlastir: "Ctrl+Shift+F10".into() }
    }
}

struct Uygulama {
    durum: Mutex<Durum>,
    kisayollar: Mutex<Kisayollar>,
}

fn ayar_dosyasi(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("kisayollar.json"))
}

fn kisayollari_oku(app: &AppHandle) -> Kisayollar {
    ayar_dosyasi(app)
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn kisayollari_yaz(app: &AppHandle, k: &Kisayollar) {
    if let Some(p) = ayar_dosyasi(app) {
        if let Some(d) = p.parent() {
            let _ = fs::create_dir_all(d);
        }
        let _ = fs::write(p, serde_json::to_string_pretty(k).unwrap_or_default());
    }
}

/// Kısayolları kaydeder (RegisterHotKey; klavye kancası değil). Döner: kaydedilemeyenlerin açıklaması.
fn kisayollari_uygula(app: &AppHandle, k: &Kisayollar) -> Vec<String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let mut hatalar = Vec::new();
    for (ad, metin) in [("Sustur", &k.sustur), ("Sağırlaştır", &k.sagirlastir)] {
        if metin.is_empty() {
            continue;
        }
        match Shortcut::from_str(metin) {
            Ok(s) => {
                if gs.register(s).is_err() {
                    hatalar.push(format!("{ad} kısayolu ({metin}) başka bir uygulamada kullanılıyor"));
                }
            }
            Err(_) => hatalar.push(format!("{ad} kısayolu ({metin}) geçersiz")),
        }
    }
    hatalar
}

// ---------------------------------------------------------------- pencere ve tepsi

fn goster(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(ANA) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn tepsiyi_guncelle(app: &AppHandle) {
    let st = app.state::<Uygulama>();
    let d = st.durum.lock().unwrap();
    let mut metin = String::from("Nahrok");
    if d.kanalda {
        metin.push_str(match (d.susturuldu, d.sagir) {
            (_, true) => " · seste (sağır)",
            (true, false) => " · seste (susturuldu)",
            _ => " · seste",
        });
    }
    if d.okunmamis > 0 {
        metin.push_str(&format!(" · {} okunmamış kanal", d.okunmamis));
    }
    if let Some(t) = app.tray_by_id(TEPSI) {
        let _ = t.set_tooltip(Some(metin));
    }
}

fn rozet_resmi(sayi: u32) -> Option<Image<'static>> {
    let bayt: &'static [u8] = match sayi {
        0 => return None,
        1 => include_bytes!("../rozet/1.png"),
        2 => include_bytes!("../rozet/2.png"),
        3 => include_bytes!("../rozet/3.png"),
        4 => include_bytes!("../rozet/4.png"),
        5 => include_bytes!("../rozet/5.png"),
        6 => include_bytes!("../rozet/6.png"),
        7 => include_bytes!("../rozet/7.png"),
        8 => include_bytes!("../rozet/8.png"),
        9 => include_bytes!("../rozet/9.png"),
        _ => include_bytes!("../rozet/9arti.png"),
    };
    Image::from_bytes(bayt).ok()
}

// ---------------------------------------------------------------- siteye açık komutlar (çizgi 3)

/// İkinci kilit: komut, izin listesinin yanı sıra yalnız sitenin kendisinden gelirse işlenir.
fn siteden_mi(w: &tauri::Webview) -> bool {
    w.label() == ANA && w.url().map(|u| ayni_site(&u)).unwrap_or(false)
}

#[tauri::command]
fn rozet(webview: tauri::Webview, app: AppHandle, sayi: u32) {
    if !siteden_mi(&webview) {
        return;
    }
    let sayi = sayi.min(99);
    app.state::<Uygulama>().durum.lock().unwrap().okunmamis = sayi;
    if let Some(w) = app.get_webview_window(ANA) {
        let _ = w.set_overlay_icon(rozet_resmi(sayi));
    }
    tepsiyi_guncelle(&app);
}

#[derive(Deserialize)]
struct SesDurumu {
    kanalda: bool,
    susturuldu: bool,
    sagir: bool,
}

#[tauri::command]
fn ses_durumu(webview: tauri::Webview, app: AppHandle, durum: SesDurumu) {
    if !siteden_mi(&webview) {
        return;
    }
    {
        let st = app.state::<Uygulama>();
        let mut d = st.durum.lock().unwrap();
        d.kanalda = durum.kanalda;
        d.susturuldu = durum.susturuldu;
        d.sagir = durum.sagir;
    }
    tepsiyi_guncelle(&app);
}

#[derive(Deserialize)]
struct KisayolDegisikligi {
    sustur: Option<String>,
    sagirlastir: Option<String>,
}

#[derive(Serialize)]
struct KisayolYaniti {
    sustur: String,
    sagirlastir: String,
    hata: Option<String>,
}

#[tauri::command]
fn kisayollar(
    webview: tauri::Webview,
    app: AppHandle,
    degisiklik: Option<KisayolDegisikligi>,
) -> Result<KisayolYaniti, String> {
    if !siteden_mi(&webview) {
        return Err("izin yok".into());
    }
    let st = app.state::<Uygulama>();
    let mut k = st.kisayollar.lock().unwrap();
    let mut hata = None;
    if let Some(d) = degisiklik {
        let mut yeni = k.clone();
        if let Some(s) = d.sustur {
            yeni.sustur = s.trim().chars().take(64).collect();
        }
        if let Some(s) = d.sagirlastir {
            yeni.sagirlastir = s.trim().chars().take(64).collect();
        }
        let gecersiz = [&yeni.sustur, &yeni.sagirlastir]
            .into_iter()
            .find(|m| !m.is_empty() && Shortcut::from_str(m).is_err());
        if let Some(m) = gecersiz {
            hata = Some(format!("{m} geçerli bir kısayol değil"));
        } else if !yeni.sustur.is_empty() && yeni.sustur == yeni.sagirlastir {
            hata = Some("İki eylem aynı kısayolu kullanamaz".into());
        } else {
            let hatalar = kisayollari_uygula(&app, &yeni);
            if hatalar.is_empty() {
                kisayollari_yaz(&app, &yeni);
                *k = yeni;
            } else {
                kisayollari_uygula(&app, &k); // eskisine dön
                hata = Some(hatalar.join("; "));
            }
        }
    }
    Ok(KisayolYaniti { sustur: k.sustur.clone(), sagirlastir: k.sagirlastir.clone(), hata })
}

// ---------------------------------------------------------------- gezinme (çizgi 2)

fn disarida_ac(app: &AppHandle, u: &Url) {
    if matches!(u.scheme(), "http" | "https" | "mailto") {
        let _ = app.opener().open_url(u.as_str(), None::<&str>);
    }
}

fn gezinmeye_izin(app: &AppHandle, u: &Url) -> bool {
    if ayni_site(u) || yerel_sayfa(u) {
        return true;
    }
    if u.scheme() == "ms-windows-store" {
        let _ = app.opener().open_url(u.as_str(), None::<&str>);
        return false;
    }
    disarida_ac(app, u);
    false
}

// ---------------------------------------------------------------- en düşük sürüm (masaustu.json)

#[derive(Deserialize)]
struct SurumBilgisi {
    #[serde(rename = "enAzSurum")]
    en_az_surum: String,
    #[serde(default)]
    store: Option<String>,
}

fn surum_parcala(s: &str) -> Vec<u64> {
    s.trim().trim_start_matches('v').split('.').map(|p| p.parse().unwrap_or(0)).collect()
}

fn surum_eski(simdiki: &str, en_az: &str) -> bool {
    let (mut a, mut b) = (surum_parcala(simdiki), surum_parcala(en_az));
    let n = a.len().max(b.len());
    a.resize(n, 0);
    b.resize(n, 0);
    a < b
}

fn surum_denetle(app: AppHandle) {
    std::thread::spawn(move || {
        let Ok(adres) = site().join("masaustu.json") else { return };
        let ajan = ureq::AgentBuilder::new().timeout(Duration::from_secs(8)).build();
        let Ok(cevap) = ajan.get(adres.as_str()).call() else { return };
        let Ok(bilgi) = cevap.into_json::<SurumBilgisi>() else { return }; // dosya yoksa site HTML döner: yok say
        if !surum_eski(SURUM, &bilgi.en_az_surum) {
            return;
        }
        let store = bilgi
            .store
            .filter(|s| s.starts_with("ms-windows-store://"))
            .unwrap_or_else(|| STORE_VARSAYILAN.into());
        let mut u = yerel("guncelle.html");
        u.query_pairs_mut().append_pair("store", &store).append_pair("surum", SURUM);
        if let Some(w) = app.get_webview_window(ANA) {
            let _ = w.navigate(u);
        }
    });
}

// ---------------------------------------------------------------- WebView2 ayrıntıları

/// Tam ekran (yayın sahnesi) pencereyi kaplasın; site açılamazsa yerel "bağlanılamadı" sayfası.
fn webview2_bagla(w: &WebviewWindow) {
    let pencere = w.clone();
    let _ = w.with_webview(move |pw| unsafe {
        use webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_WEB_ERROR_STATUS_OPERATION_CANCELED;
        use webview2_com::{ContainsFullScreenElementChangedEventHandler, NavigationCompletedEventHandler};
        let Ok(cw) = pw.controller().CoreWebView2() else { return };
        let mut belirtec = Default::default();

        let p = pencere.clone();
        let _ = cw.add_ContainsFullScreenElementChanged(
            &ContainsFullScreenElementChangedEventHandler::create(Box::new(move |kaynak, _| {
                if let Some(k) = kaynak {
                    let mut tam = windows_core::BOOL::default();
                    k.ContainsFullScreenElement(&mut tam)?;
                    let _ = p.set_fullscreen(tam.as_bool());
                }
                Ok(())
            })),
            &mut belirtec,
        );

        let p = pencere.clone();
        let _ = cw.add_NavigationCompleted(
            &NavigationCompletedEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let mut basarili = windows_core::BOOL::default();
                args.IsSuccess(&mut basarili)?;
                if basarili.as_bool() {
                    return Ok(());
                }
                let mut durum = Default::default();
                args.WebErrorStatus(&mut durum)?;
                if durum == COREWEBVIEW2_WEB_ERROR_STATUS_OPERATION_CANCELED {
                    return Ok(());
                }
                if p.url().map(|u| ayni_site(&u)).unwrap_or(true) {
                    let mut u = yerel("cevrimdisi.html");
                    u.query_pairs_mut().append_pair("site", site().as_str());
                    let _ = p.navigate(u);
                }
                Ok(())
            })),
            &mut belirtec,
        );
    });
}

// ---------------------------------------------------------------- başlangıç

fn kopru_betigi() -> String {
    include_str!("kopru.js")
        .replace("__NAHROK_KOKEN__", &site().origin().ascii_serialization())
        .replace("__NAHROK_SURUM__", SURUM)
}

fn main() {
    tauri::Builder::default()
        // Tek kopya: ikinci açılış var olan pencereyi öne getirir (ilk eklenti olmalı)
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| goster(app)))
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, kisayol, olay| {
                    if olay.state() != ShortcutState::Pressed {
                        return;
                    }
                    let k = app.state::<Uygulama>().kisayollar.lock().unwrap().clone();
                    let esit = |m: &str| Shortcut::from_str(m).map(|s| s.id() == kisayol.id()).unwrap_or(false);
                    let eylem = if esit(&k.sustur) {
                        "sustur"
                    } else if esit(&k.sagirlastir) {
                        "sagirlastir"
                    } else {
                        return;
                    };
                    if let Some(w) = app.get_webview_window(ANA) {
                        let _ = w.eval(format!("window.__nahrokKisayol&&window.__nahrokKisayol('{eylem}')"));
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![rozet, ses_durumu, kisayollar])
        .setup(|app| {
            let h = app.handle().clone();
            let k = kisayollari_oku(&h);
            app.manage(Uygulama { durum: Mutex::new(Durum::default()), kisayollar: Mutex::new(k.clone()) });
            kisayollari_uygula(&h, &k);

            let (h1, h2) = (h.clone(), h.clone());
            let w = WebviewWindowBuilder::new(app, ANA, WebviewUrl::External(site()))
                .title("Nahrok")
                .inner_size(1280.0, 820.0)
                .min_inner_size(420.0, 520.0)
                .initialization_script(kopru_betigi())
                .on_navigation(move |u| gezinmeye_izin(&h1, u))
                .on_new_window(move |u, _| {
                    // Yeni pencere açılmaz: aynı siteyse ana pencerede, değilse sistem tarayıcısında
                    if ayni_site(&u) {
                        if let Some(w) = h2.get_webview_window(ANA) {
                            let _ = w.navigate(u);
                        }
                    } else {
                        disarida_ac(&h2, &u);
                    }
                    NewWindowResponse::Deny
                })
                .on_permission_request(|wv, tur| {
                    let site_mi = wv.url().map(|u| ayni_site(&u)).unwrap_or(false);
                    match tur {
                        PermissionKind::Microphone | PermissionKind::Autoplay if site_mi => PermissionResponse::Allow,
                        _ => PermissionResponse::Deny,
                    }
                })
                .build()?;
            webview2_bagla(&w);

            // Seste iken pencereyi kapatmak uygulamayı tepsiye indirir; değilse uygulama kapanır
            let (h3, w3) = (h.clone(), w.clone());
            w.on_window_event(move |olay| {
                if let WindowEvent::CloseRequested { api, .. } = olay {
                    if h3.state::<Uygulama>().durum.lock().unwrap().kanalda {
                        api.prevent_close();
                        let _ = w3.hide();
                    }
                }
            });

            let ac = MenuItem::with_id(app, "ac", "Nahrok'u aç", true, None::<&str>)?;
            let cik = MenuItem::with_id(app, "cik", "Çıkış", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&ac, &cik])?;
            TrayIconBuilder::with_id(TEPSI)
                .icon(app.default_window_icon().cloned().expect("uygulama simgesi"))
                .tooltip("Nahrok")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, e| match e.id().as_ref() {
                    "ac" => goster(app),
                    "cik" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tepsi, e| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                        goster(tepsi.app_handle());
                    }
                })
                .build(app)?;

            surum_denetle(h);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Nahrok başlatılamadı");
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn surum_karsilastirma() {
        assert!(surum_eski("1.0.0", "1.0.1"));
        assert!(surum_eski("1.9.0", "1.10.0"));
        assert!(!surum_eski("1.0.0", "1.0.0"));
        assert!(!surum_eski("1.2.0", "1.1.9"));
        assert!(!surum_eski("1.0.0", "1"));
        assert!(surum_eski("1.0.0", "v2"));
    }

    #[test]
    fn varsayilan_kisayollar_gecerli() {
        let k = Kisayollar::default();
        assert!(Shortcut::from_str(&k.sustur).is_ok());
        assert!(Shortcut::from_str(&k.sagirlastir).is_ok());
    }

    #[test]
    fn yerel_ve_site_ayrimi() {
        assert!(ayni_site(&"https://ekorhan.site/chat".parse().unwrap()));
        assert!(!ayni_site(&"https://ekorhan.site.kotu.com/".parse().unwrap()));
        assert!(!ayni_site(&"http://ekorhan.site/".parse().unwrap()));
        assert!(yerel_sayfa(&yerel("cevrimdisi.html")));
        assert!(!yerel_sayfa(&"https://tauri.localhost.kotu.com/".parse().unwrap()));
    }
}
