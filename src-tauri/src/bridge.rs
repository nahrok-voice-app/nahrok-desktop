//! Çizgi 3: sitenin uygulamadan isteyebileceği kısa liste (window.nahrokDesktop, bridge.js). Komutların izni üç
//! yerde verilir: build.rs (komut adları), capabilities/site.json (yalnız sitenin adresi) ve burada köken denetimi.

use serde::Deserialize;
use tauri::{AppHandle, Manager, Webview};

use crate::{navigation, settings_window, tray, AppState, MAIN_WINDOW, VERSION};

/// Ana penceredeki sayfa her yüklendiğinde, sayfanın kendi betiklerinden önce çalışan köprü betiği
pub fn script() -> String {
    include_str!("bridge.js")
        .replace("__NAHROK_SITE_ORIGIN__", &navigation::site_url().origin().ascii_serialization())
        .replace("__NAHROK_VERSION__", VERSION)
}

/// İkinci kilit: komut, izin listesinin yanı sıra yalnız ana penceredeki siteden gelirse işlenir.
fn from_site(webview: &Webview) -> bool {
    webview.label() == MAIN_WINDOW && webview.url().map(|u| navigation::is_site(&u)).unwrap_or(false)
}

/// Okunmamış metin kanalı sayısı: görev çubuğu rozeti ve tepsi ipucu
#[tauri::command]
pub fn set_badge(webview: Webview, app: AppHandle, count: u32) {
    if !from_site(&webview) {
        return;
    }
    *app.state::<AppState>().unread.lock().unwrap() = count.min(99);
    tray::refresh(&app);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceStateArgs {
    in_voice: bool,
    muted: bool,
    deafened: bool,
}

/// Sesli kanal durumu: pencere kapanınca seste kalma, tepsi menüsü ve ipucu buna göre ayarlanır
#[tauri::command]
pub fn set_voice_state(webview: Webview, app: AppHandle, state: VoiceStateArgs) {
    if !from_site(&webview) {
        return;
    }
    *app.state::<AppState>().voice.lock().unwrap() =
        crate::VoiceState { in_voice: state.in_voice, muted: state.muted, deafened: state.deafened };
    tray::refresh(&app);
}

/// Ayarlar penceresini açar. Pencere oluşturduğu için async: Windows'ta eşzamanlı komuttan pencere açmak kilitlenir.
#[tauri::command]
pub async fn open_settings(webview: Webview, app: AppHandle) {
    if from_site(&webview) {
        settings_window::open(&app);
    }
}
