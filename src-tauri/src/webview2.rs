//! WebView2'ye özgü iki ayar: tam ekran (yayın sahnesi) pencereyi kaplar; site açılamazsa yerel "bağlanılamadı"
//! sayfası gösterilir.

use tauri::WebviewWindow;
use webview2_com::{
    ContainsFullScreenElementChangedEventHandler,
    Microsoft::Web::WebView2::Win32::COREWEBVIEW2_WEB_ERROR_STATUS_OPERATION_CANCELED, NavigationCompletedEventHandler,
};

use crate::navigation;

pub fn attach(window: &WebviewWindow) {
    let window = window.clone();
    let target = window.clone();
    let _ = target.with_webview(move |platform| unsafe {
        let Ok(core) = platform.controller().CoreWebView2() else { return };
        let mut token = Default::default();

        let fullscreen_window = window.clone();
        let _ = core.add_ContainsFullScreenElementChanged(
            &ContainsFullScreenElementChangedEventHandler::create(Box::new(move |sender, _| {
                if let Some(sender) = sender {
                    let mut fullscreen = windows_core::BOOL::default();
                    sender.ContainsFullScreenElement(&mut fullscreen)?;
                    let _ = fullscreen_window.set_fullscreen(fullscreen.as_bool());
                }
                Ok(())
            })),
            &mut token,
        );

        let offline_window = window.clone();
        let _ = core.add_NavigationCompleted(
            &NavigationCompletedEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let mut success = windows_core::BOOL::default();
                args.IsSuccess(&mut success)?;
                if success.as_bool() {
                    return Ok(());
                }
                let mut status = Default::default();
                args.WebErrorStatus(&mut status)?;
                if status == COREWEBVIEW2_WEB_ERROR_STATUS_OPERATION_CANCELED {
                    return Ok(());
                }
                // Yalnız sitenin kendisi açılamadıysa (yerel sayfalarda döngüye girmemek için)
                if offline_window.url().map(|u| navigation::is_site(&u)).unwrap_or(true) {
                    let mut page = navigation::local_page("offline.html");
                    page.query_pairs_mut().append_pair("site", navigation::site_url().as_str());
                    let _ = offline_window.navigate(page);
                }
                Ok(())
            })),
            &mut token,
        );
    });
}
