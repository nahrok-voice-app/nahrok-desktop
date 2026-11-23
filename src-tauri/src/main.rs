// Nahrok masaüstü uygulaması. Kurallar PLAN.md "Kesin çizgiler"dedir; modüller onları uygular:
//   navigation      çizgi 2: tek adres, gezinme ve izinler
//   bridge          çizgi 3: siteye açık kısa komut listesi
//   settings        kullanıcı ayarları ve dosyası
//   settings_window uygulamanın ayarlar penceresi ve komutları
//   shortcuts       oyundayken çalışan mikrofon / ses kısayolları
//   startup         bilgisayar açılınca başlatma (MSIX başlangıç görevi; çizgi 7: varsayılan kapalı)
//   tray            tepsi simgesi, menüsü ve görev çubuğu rozeti
//   version_check   en düşük sürüm denetimi
//   webview2        tam ekran ve "bağlanılamadı" sayfası
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridge;
mod navigation;
mod settings;
mod settings_window;
mod shortcuts;
mod startup;
mod tray;
mod version_check;
mod webview2;

use std::sync::Mutex;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_window_state::StateFlags;

pub const MAIN_WINDOW: &str = "main";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Siteden gelen ses durumu (bridge::set_voice_state)
#[derive(Default, Clone, Copy)]
pub struct VoiceState {
    pub in_voice: bool,
    pub muted: bool,
    pub deafened: bool,
}

pub struct AppState {
    pub voice: Mutex<VoiceState>,
    pub unread: Mutex<u32>,
    pub settings: Mutex<settings::Settings>,
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN_WINDOW) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn main() {
    tauri::Builder::default()
        // Tek kopya: ikinci açılış var olan pencereyi öne getirir (ilk eklenti olmalı)
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_main_window(app)))
        .plugin(tauri_plugin_opener::init())
        // Pencerenin boyutu ve yeri hatırlanır; görünürlük hatırlanmaz (tepside başlatma ayarı karar verir)
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(StateFlags::all() - StateFlags::VISIBLE)
                .with_denylist(&[settings_window::LABEL])
                .build(),
        )
        .plugin(shortcuts::plugin())
        .invoke_handler(tauri::generate_handler![
            bridge::set_badge,
            bridge::set_voice_state,
            bridge::open_settings,
            settings_window::get_settings,
            settings_window::update_settings,
            settings_window::set_start_at_login,
            settings_window::open_windows_startup_settings,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let user_settings = settings::load(&handle);
            let start_hidden = user_settings.start_hidden && startup::launched_at_login();
            app.manage(AppState {
                voice: Mutex::new(VoiceState::default()),
                unread: Mutex::new(0),
                settings: Mutex::new(user_settings.clone()),
            });
            shortcuts::register(&handle, &user_settings);

            let (nav_app, popup_app) = (handle.clone(), handle.clone());
            let window = WebviewWindowBuilder::new(app, MAIN_WINDOW, WebviewUrl::External(navigation::site_url()))
                .title("Nahrok")
                .inner_size(1280.0, 820.0)
                .min_inner_size(420.0, 520.0)
                .visible(!start_hidden)
                .initialization_script(bridge::script())
                .on_navigation(move |url| navigation::allow_main_navigation(&nav_app, url))
                .on_new_window(move |url, _| navigation::handle_new_window(&popup_app, url))
                .on_permission_request(navigation::main_window_permission)
                .build()?;
            webview2::attach(&window);

            // Pencereyi kapatmak: seste iken ya da "kapatınca tepside çalışsın" açıksa tepsiye iner, değilse çıkar
            let close_app = handle.clone();
            let close_window = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    let state = close_app.state::<AppState>();
                    let keep_running =
                        state.voice.lock().unwrap().in_voice || state.settings.lock().unwrap().close_to_tray;
                    if keep_running {
                        api.prevent_close();
                        let _ = close_window.hide();
                    } else {
                        close_app.exit(0);
                    }
                }
            });

            tray::create(app)?;
            version_check::run(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Nahrok başlatılamadı");
}
