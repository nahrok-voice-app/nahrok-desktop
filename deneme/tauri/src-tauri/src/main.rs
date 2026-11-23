// Nahrok masaüstü, Tauri denemesi. PLAN.md çizgi 1-2: siteyi yükler, web'e komut vermez.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{WebviewUrl, WebviewWindowBuilder};

fn main() {
    // Tek sabit adres. Denemede: NAHROK_URL=https://... ile verilir.
    let site = std::env::var("NAHROK_URL").unwrap_or_else(|_| "http://localhost:5173".into());
    let site: tauri::Url = site.parse().expect("NAHROK_URL geçersiz");
    let origin = site.origin();

    tauri::Builder::default()
        .setup(move |app| {
            let izinli = origin.clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(site.clone()))
                .title("Nahrok")
                .inner_size(1280.0, 800.0)
                // Başka adrese gezinme engellenir (dış bağlantıları tarayıcıda açma sonraki adım).
                .on_navigation(move |url| url.origin() == izinli)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Nahrok başlatılamadı");
}
