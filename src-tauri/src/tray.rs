//! Tepsi simgesi ve menüsü, görev çubuğu rozeti. Ses durumu ve okunmamış sayısı siteden gelir (bridge.rs).

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Wry,
};

use crate::{
    settings_window,
    shortcuts::{self, ShortcutAction},
    show_main_window, AppState, VoiceState, MAIN_WINDOW,
};

const TRAY_ID: &str = "tray";

/// Seste değişen menü satırları
struct TrayMenu {
    toggle_mute: MenuItem<Wry>,
    toggle_deafen: MenuItem<Wry>,
}

pub fn create(app: &App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Nahrok'u aç", true, None::<&str>)?;
    let toggle_mute = MenuItem::with_id(app, "toggleMute", "Mikrofonu kapat", false, None::<&str>)?;
    let toggle_deafen = MenuItem::with_id(app, "toggleDeafen", "Sesi kapat", false, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Ayarlar", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Çıkış", true, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[&show, &separator1, &toggle_mute, &toggle_deafen, &separator2, &settings, &quit],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().cloned().expect("uygulama simgesi"))
        .tooltip("Nahrok")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "toggleMute" => shortcuts::send_to_site(app, ShortcutAction::ToggleMute),
            "toggleDeafen" => shortcuts::send_to_site(app, ShortcutAction::ToggleDeafen),
            "settings" => {
                // Pencere ana iş parçacığının dışında oluşturulur (Windows'ta olay işleyicisinden açmak kilitlenebilir)
                let app = app.clone();
                tauri::async_runtime::spawn(async move { settings_window::open(&app) });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    app.manage(TrayMenu { toggle_mute, toggle_deafen });
    Ok(())
}

/// Ses durumu ya da okunmamış sayısı değişince: tepsi ipucu, menü ve görev çubuğu rozeti
pub fn refresh(app: &AppHandle) {
    let state = app.state::<AppState>();
    let voice = *state.voice.lock().unwrap();
    let unread = *state.unread.lock().unwrap();

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(tooltip(voice, unread)));
    }
    if let Some(menu) = app.try_state::<TrayMenu>() {
        let _ = menu.toggle_mute.set_text(if voice.muted { "Mikrofonu aç" } else { "Mikrofonu kapat" });
        let _ = menu.toggle_mute.set_enabled(voice.in_voice);
        let _ = menu.toggle_deafen.set_text(if voice.deafened { "Sesi aç" } else { "Sesi kapat" });
        let _ = menu.toggle_deafen.set_enabled(voice.in_voice);
    }
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.set_overlay_icon(badge_image(unread));
    }
}

fn tooltip(voice: VoiceState, unread: u32) -> String {
    let mut lines = vec![String::from("Nahrok")];
    if voice.in_voice {
        lines.push(
            match (voice.muted, voice.deafened) {
                (_, true) => "Sesli kanaldasın, sesin kapalı",
                (true, false) => "Sesli kanaldasın, mikrofonun kapalı",
                _ => "Sesli kanaldasın",
            }
            .into(),
        );
    }
    if unread > 0 {
        let count = if unread >= 99 { "99+".to_string() } else { unread.to_string() };
        lines.push(format!("{count} kanalda okunmamış mesaj var"));
    }
    lines.join("\n")
}

fn badge_image(count: u32) -> Option<Image<'static>> {
    let bytes: &'static [u8] = match count {
        0 => return None,
        1 => include_bytes!("../badges/1.png"),
        2 => include_bytes!("../badges/2.png"),
        3 => include_bytes!("../badges/3.png"),
        4 => include_bytes!("../badges/4.png"),
        5 => include_bytes!("../badges/5.png"),
        6 => include_bytes!("../badges/6.png"),
        7 => include_bytes!("../badges/7.png"),
        8 => include_bytes!("../badges/8.png"),
        9 => include_bytes!("../badges/9.png"),
        _ => include_bytes!("../badges/9plus.png"),
    };
    Image::from_bytes(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_describes_voice_and_unread() {
        assert_eq!(tooltip(VoiceState::default(), 0), "Nahrok");
        let muted = VoiceState { in_voice: true, muted: true, deafened: false };
        assert_eq!(tooltip(muted, 3), "Nahrok\nSesli kanaldasın, mikrofonun kapalı\n3 kanalda okunmamış mesaj var");
        let deafened = VoiceState { in_voice: true, muted: true, deafened: true };
        assert_eq!(tooltip(deafened, 120), "Nahrok\nSesli kanaldasın, sesin kapalı\n99+ kanalda okunmamış mesaj var");
    }

    #[test]
    fn every_badge_image_decodes() {
        for n in 1..=10 {
            assert!(badge_image(n).is_some(), "rozet {n}");
        }
        assert!(badge_image(0).is_none());
    }
}
