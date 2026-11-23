//! En düşük sürüm denetimi. Site /desktop.json yayımlar: { "minVersion": "1.1.0", "storeUrl": "ms-windows-store://…" }.
//! Uygulama bundan eskiyse site yerine "güncelleme gerekli" sayfası açılır. Dosya yoksa site HTML döndürür; o zaman
//! denetim hiçbir şey yapmaz.

use std::time::Duration;

use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::{navigation, MAIN_WINDOW, VERSION};

const DEFAULT_STORE_URL: &str = "ms-windows-store://downloadsandupdates";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopInfo {
    min_version: String,
    #[serde(default)]
    store_url: Option<String>,
}

fn parse_version(text: &str) -> Vec<u64> {
    text.trim().trim_start_matches('v').split('.').map(|part| part.parse().unwrap_or(0)).collect()
}

fn is_older(current: &str, minimum: &str) -> bool {
    let (mut a, mut b) = (parse_version(current), parse_version(minimum));
    let len = a.len().max(b.len());
    a.resize(len, 0);
    b.resize(len, 0);
    a < b
}

pub fn run(app: AppHandle) {
    std::thread::spawn(move || {
        let Ok(url) = navigation::site_url().join("desktop.json") else { return };
        let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(8)).build();
        let Ok(response) = agent.get(url.as_str()).call() else { return };
        let Ok(info) = response.into_json::<DesktopInfo>() else { return };
        if !is_older(VERSION, &info.min_version) {
            return;
        }
        let store_url = info
            .store_url
            .filter(|u| u.starts_with("ms-windows-store://"))
            .unwrap_or_else(|| DEFAULT_STORE_URL.into());
        let mut page = navigation::local_page("update.html");
        page.query_pairs_mut().append_pair("store", &store_url).append_pair("version", VERSION);
        if let Some(w) = app.get_webview_window(MAIN_WINDOW) {
            let _ = w.navigate(page);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_numerically() {
        assert!(is_older("1.0.0", "1.0.1"));
        assert!(is_older("1.9.0", "1.10.0"));
        assert!(!is_older("1.0.0", "1.0.0"));
        assert!(!is_older("1.2.0", "1.1.9"));
        assert!(!is_older("1.0.0", "1"));
        assert!(is_older("1.0.0", "v2"));
    }
}
