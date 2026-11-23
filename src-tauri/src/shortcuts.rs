//! Oyundayken de çalışan mikrofon / ses kısayolları. Windows'un RegisterHotKey'i kullanılır, klavye kancası değil
//! (PLAN.md v1 "Yok": bas-konuş). Basılınca site bilgilendirilir; susturmayı site yapar.

use std::str::FromStr;

use tauri::{plugin::TauriPlugin, AppHandle, Manager, Wry};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::{settings::Settings, AppState, MAIN_WINDOW};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShortcutAction {
    ToggleMute,
    ToggleDeafen,
}

impl ShortcutAction {
    /// bridge.js'deki eylem adı
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ToggleMute => "toggleMute",
            Self::ToggleDeafen => "toggleDeafen",
        }
    }
}

/// Eylemi siteye iletir (kısayol ya da tepsi menüsü). Site dinlemiyorsa hiçbir şey olmaz.
pub fn send_to_site(app: &AppHandle, action: ShortcutAction) {
    if let Some(w) = app.get_webview_window(MAIN_WINDOW) {
        let _ = w.eval(format!("window.__nahrokShortcut && window.__nahrokShortcut('{}')", action.as_str()));
    }
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, pressed, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            let settings = app.state::<AppState>().settings.lock().unwrap().clone();
            let matches = |text: &str| Shortcut::from_str(text).map(|s| s.id() == pressed.id()).unwrap_or(false);
            if matches(&settings.toggle_mute) {
                send_to_site(app, ShortcutAction::ToggleMute);
            } else if matches(&settings.toggle_deafen) {
                send_to_site(app, ShortcutAction::ToggleDeafen);
            }
        })
        .build()
}

/// Kısayolları yeniden kaydeder. Döner: kaydedilemeyenlerin açıklaması (ör. başka uygulama kullanıyor).
pub fn register(app: &AppHandle, settings: &Settings) -> Vec<String> {
    let manager = app.global_shortcut();
    let _ = manager.unregister_all();
    let mut errors = Vec::new();
    for (name, text) in [("Mikrofon", &settings.toggle_mute), ("Ses", &settings.toggle_deafen)] {
        if text.is_empty() {
            continue;
        }
        match Shortcut::from_str(text) {
            Ok(shortcut) => {
                if manager.register(shortcut).is_err() {
                    errors.push(format!("{name} kısayolunu ({text}) başka bir uygulama kullanıyor."));
                }
            }
            Err(_) => errors.push(format!("{name} kısayolu ({text}) geçerli değil.")),
        }
    }
    errors
}
