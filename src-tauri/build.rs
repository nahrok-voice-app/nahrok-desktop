// Uygulamanın komutları burada sabittir. Hangisini kimin çağırabileceğini capabilities/ belirler:
// site.json siteye üçünü (PLAN.md çizgi 3; listeye ekleme PLAN.md'yi değiştirmek demektir), settings.json ayarlar
// penceresine dördünü açar.
const COMMANDS: &[&str] = &[
    // site (bridge.rs)
    "set_badge",
    "set_voice_state",
    "open_settings",
    // ayarlar penceresi (settings_window.rs)
    "get_settings",
    "update_settings",
    "set_start_at_login",
    "open_windows_startup_settings",
];

fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)))
        .expect("tauri-build başarısız");
}
