//! Kullanıcı ayarları: %APPDATA%\com.nahrok.desktop\settings.json. Yalnız ayarlar penceresi değiştirir.

use std::{fs, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::Shortcut;

const FILE_NAME: &str = "settings.json";
const MAX_SHORTCUT_LEN: usize = 64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// Mikrofonu aç / kapat kısayolu ("" = yok)
    pub toggle_mute: String,
    /// Sesi aç / kapat kısayolu ("" = yok)
    pub toggle_deafen: String,
    /// Pencereyi kapatınca seste olmasa da tepside çalışmaya devam et
    pub close_to_tray: bool,
    /// Bilgisayar açılınca pencereyi gösterme, tepside başla
    pub start_hidden: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // Oyunlarda ve tarayıcılarda nadiren kullanılan birleşimler
            toggle_mute: "Ctrl+Shift+F9".into(),
            toggle_deafen: "Ctrl+Shift+F10".into(),
            close_to_tray: false,
            start_hidden: true,
        }
    }
}

/// Ayarlar penceresinden gelen değişiklik: yalnız gönderilen alanlar değişir
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    pub toggle_mute: Option<String>,
    pub toggle_deafen: Option<String>,
    pub close_to_tray: Option<bool>,
    pub start_hidden: Option<bool>,
}

impl Settings {
    /// Değişikliği uygular ve doğrular. Hata metni ayarlar penceresinde gösterilir.
    pub fn with_patch(&self, patch: SettingsPatch) -> Result<Settings, String> {
        let mut next = self.clone();
        if let Some(value) = patch.toggle_mute {
            next.toggle_mute = clean_shortcut(&value);
        }
        if let Some(value) = patch.toggle_deafen {
            next.toggle_deafen = clean_shortcut(&value);
        }
        if let Some(value) = patch.close_to_tray {
            next.close_to_tray = value;
        }
        if let Some(value) = patch.start_hidden {
            next.start_hidden = value;
        }
        for shortcut in [&next.toggle_mute, &next.toggle_deafen] {
            if !shortcut.is_empty() && Shortcut::from_str(shortcut).is_err() {
                return Err(format!("{shortcut} geçerli bir kısayol değil."));
            }
        }
        if !next.toggle_mute.is_empty() && next.toggle_mute == next.toggle_deafen {
            return Err("İki eylem aynı kısayolu kullanamaz.".into());
        }
        Ok(next)
    }

    pub fn same_shortcuts(&self, other: &Settings) -> bool {
        self.toggle_mute == other.toggle_mute && self.toggle_deafen == other.toggle_deafen
    }
}

fn clean_shortcut(value: &str) -> String {
    value.trim().chars().take(MAX_SHORTCUT_LEN).collect()
}

fn file_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|dir| dir.join(FILE_NAME))
}

/// Dosya yoksa ya da bozuksa varsayılanlar
pub fn load(app: &AppHandle) -> Settings {
    file_path(app)
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &Settings) {
    let Some(path) = file_path(app) else { return };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shortcuts_are_valid() {
        let s = Settings::default();
        assert!(Shortcut::from_str(&s.toggle_mute).is_ok());
        assert!(Shortcut::from_str(&s.toggle_deafen).is_ok());
        assert!(!s.close_to_tray);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let s: Settings = serde_json::from_str(r#"{ "toggleMute": "Alt+M" }"#).unwrap();
        assert_eq!(s.toggle_mute, "Alt+M");
        assert_eq!(s.toggle_deafen, Settings::default().toggle_deafen);
        assert!(s.start_hidden);
    }

    #[test]
    fn patch_changes_only_given_fields() {
        let s = Settings::default();
        let next = s.with_patch(SettingsPatch { close_to_tray: Some(true), ..Default::default() }).unwrap();
        assert!(next.close_to_tray);
        assert!(next.same_shortcuts(&s));

        let next = s.with_patch(SettingsPatch { toggle_mute: Some("  Ctrl+Alt+M ".into()), ..Default::default() }).unwrap();
        assert_eq!(next.toggle_mute, "Ctrl+Alt+M");
        assert!(!next.same_shortcuts(&s));

        let next = s.with_patch(SettingsPatch { toggle_deafen: Some(String::new()), ..Default::default() }).unwrap();
        assert_eq!(next.toggle_deafen, "");
    }

    #[test]
    fn invalid_or_duplicate_shortcuts_are_rejected() {
        let s = Settings::default();
        let bad = s.with_patch(SettingsPatch { toggle_mute: Some("Ctrl+Shift+Bilinmeyen".into()), ..Default::default() });
        assert!(bad.is_err());
        let same = s.with_patch(SettingsPatch { toggle_mute: Some("Ctrl+Shift+F10".into()), ..Default::default() });
        assert_eq!(same.unwrap_err(), "İki eylem aynı kısayolu kullanamaz.");
    }
}
