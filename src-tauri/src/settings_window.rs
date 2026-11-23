//! Ayarlar penceresi: uygulamanın kendi yerel sayfası (ui/settings.html). Buradaki komutları yalnız bu pencere
//! çağırabilir (capabilities/settings.json + pencere ve adres denetimi); site çağıramaz.

use serde::Serialize;
use tauri::{
    webview::{NewWindowResponse, PermissionResponse},
    AppHandle, Manager, Webview, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_opener::OpenerExt;

use crate::{
    navigation,
    settings::{self, Settings, SettingsPatch},
    shortcuts,
    startup::{self, StartupState},
    AppState, VERSION,
};

pub const LABEL: &str = "settings";
const DENIED: &str = "Bu komut yalnız ayarlar penceresinden çağrılabilir.";

/// Pencere açıksa öne getirir, değilse açar. Ana iş parçacığında eşzamanlı komuttan çağrılmamalı (Windows'ta kilitlenir).
pub fn open(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("settings.html".into()))
        .title("Nahrok ayarları")
        .inner_size(500.0, 820.0)
        .min_inner_size(380.0, 480.0)
        .center()
        .on_navigation(navigation::is_local_page)
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_permission_request(|_, _| PermissionResponse::Deny)
        .build();
}

fn from_settings_window(webview: &Webview) -> bool {
    webview.label() == LABEL && webview.url().map(|u| navigation::is_local_page(&u)).unwrap_or(false)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    #[serde(flatten)]
    settings: Settings,
    start_at_login: StartupState,
    version: &'static str,
}

fn view(app: &AppHandle) -> SettingsView {
    SettingsView {
        settings: app.state::<AppState>().settings.lock().unwrap().clone(),
        start_at_login: startup::state(),
        version: VERSION,
    }
}

#[tauri::command]
pub async fn get_settings(webview: Webview, app: AppHandle) -> Result<SettingsView, String> {
    if !from_settings_window(&webview) {
        return Err(DENIED.into());
    }
    Ok(view(&app))
}

#[derive(Serialize)]
pub struct UpdateResult {
    settings: SettingsView,
    error: Option<String>,
}

#[tauri::command]
pub async fn update_settings(webview: Webview, app: AppHandle, patch: SettingsPatch) -> Result<UpdateResult, String> {
    if !from_settings_window(&webview) {
        return Err(DENIED.into());
    }
    // Kilit kısayol kaydı sırasında tutulmaz: kısayol işleyicisi ana iş parçacığında aynı kilidi ister.
    let current = app.state::<AppState>().settings.lock().unwrap().clone();
    let error = match current.with_patch(patch) {
        Err(message) => Some(message),
        Ok(next) => {
            let errors = if next.same_shortcuts(&current) { Vec::new() } else { shortcuts::register(&app, &next) };
            if errors.is_empty() {
                settings::save(&app, &next);
                *app.state::<AppState>().settings.lock().unwrap() = next;
                None
            } else {
                shortcuts::register(&app, &current); // eski kısayollara dön
                Some(errors.join(" "))
            }
        }
    };
    Ok(UpdateResult { settings: view(&app), error })
}

#[tauri::command]
pub async fn set_start_at_login(webview: Webview, enabled: bool) -> Result<StartupState, String> {
    if !from_settings_window(&webview) {
        return Err(DENIED.into());
    }
    Ok(startup::set_enabled(enabled))
}

/// Windows Ayarlar > Uygulamalar > Başlangıç (kullanıcı orada kapattıysa yalnız orada açılabilir)
#[tauri::command]
pub fn open_windows_startup_settings(webview: Webview, app: AppHandle) -> Result<(), String> {
    if !from_settings_window(&webview) {
        return Err(DENIED.into());
    }
    app.opener().open_url("ms-settings:startupapps", None::<&str>).map_err(|e| e.to_string())
}
