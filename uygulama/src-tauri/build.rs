// Siteye açılan komutlar burada sabittir (PLAN.md çizgi 3). Listeye ekleme, PLAN.md'yi değiştirmek demektir.
fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["rozet", "ses_durumu", "kisayollar"])),
    )
    .expect("tauri-build başarısız");
}
