//! Çizgi 2: site, uygulamanın gözünde güvenilmezdir. Yalnız tek adres yüklenir; başka adrese gezinme ve yeni pencere
//! engellenir, dış bağlantılar sistem tarayıcısında açılır. İzin olarak yalnız mikrofon ve (sesli kanal sesi için)
//! otomatik oynatma verilir; ekran yakalama WebView2'nin kendi seçicisiyle olur (çizgi 4). Sertifika hatası atlanmaz
//! (WebView2 varsayılanı, burada değiştirilmez).

use tauri::{
    webview::{NewWindowResponse, PermissionKind, PermissionResponse},
    AppHandle, Manager, Url, Webview, Wry,
};
use tauri_plugin_opener::OpenerExt;

use crate::MAIN_WINDOW;

const SITE: &str = "https://ekorhan.site";
const LOCAL_HOST: &str = "tauri.localhost";

/// Yüklenen tek adres. Hata ayıklama derlemesinde duman testi için NAHROK_URL ile değiştirilebilir; yayında değiştirilemez.
pub fn site_url() -> Url {
    #[cfg(debug_assertions)]
    if let Some(url) = std::env::var("NAHROK_URL").ok().and_then(|u| u.parse().ok()) {
        return url;
    }
    SITE.parse().expect("SITE geçerli bir adres")
}

pub fn is_site(url: &Url) -> bool {
    url.origin() == site_url().origin()
}

/// Uygulamanın kendi sayfaları (ui/): Windows'ta http://tauri.localhost altında sunulur.
pub fn is_local_page(url: &Url) -> bool {
    url.host_str() == Some(LOCAL_HOST) && matches!(url.scheme(), "http" | "https")
}

pub fn local_page(page: &str) -> Url {
    Url::parse(&format!("http://{LOCAL_HOST}/")).unwrap().join(page).unwrap()
}

pub fn open_external(app: &AppHandle, url: &Url) {
    if matches!(url.scheme(), "http" | "https" | "mailto") {
        let _ = app.opener().open_url(url.as_str(), None::<&str>);
    }
}

pub fn allow_main_navigation(app: &AppHandle, url: &Url) -> bool {
    if is_site(url) || is_local_page(url) {
        return true;
    }
    // "Güncelleme gerekli" sayfasındaki Store bağlantısı
    if url.scheme() == "ms-windows-store" {
        let _ = app.opener().open_url(url.as_str(), None::<&str>);
        return false;
    }
    open_external(app, url);
    false
}

/// Yeni pencere açılmaz: aynı siteyse ana pencerede, değilse sistem tarayıcısında açılır.
pub fn handle_new_window(app: &AppHandle, url: Url) -> NewWindowResponse<Wry> {
    if is_site(&url) {
        if let Some(w) = app.get_webview_window(MAIN_WINDOW) {
            let _ = w.navigate(url);
        }
    } else {
        open_external(app, &url);
    }
    NewWindowResponse::Deny
}

pub fn main_window_permission(webview: Webview, kind: PermissionKind) -> PermissionResponse {
    let from_site = webview.url().map(|u| is_site(&u)).unwrap_or(false);
    match kind {
        PermissionKind::Microphone | PermissionKind::Autoplay if from_site => PermissionResponse::Allow,
        _ => PermissionResponse::Deny,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_and_local_pages_are_told_apart() {
        assert!(is_site(&"https://ekorhan.site/chat".parse().unwrap()));
        assert!(!is_site(&"https://ekorhan.site.example.com/".parse().unwrap()));
        assert!(!is_site(&"http://ekorhan.site/".parse().unwrap()));
        assert!(is_local_page(&local_page("offline.html")));
        assert!(!is_local_page(&"https://tauri.localhost.example.com/".parse().unwrap()));
    }
}
