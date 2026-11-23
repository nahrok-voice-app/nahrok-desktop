//! Bilgisayar açılınca başlatma: MSIX paketindeki başlangıç görevi (msix/AppxManifest.xml, TaskId "NahrokStartup").
//! Varsayılan kapalıdır (PLAN.md çizgi 7); kullanıcı ayarlar penceresinden ya da Windows ayarlarından açar. Paketsiz
//! çalışan derlemede (hata ayıklama, duman testi) başlangıç görevi yoktur ve durum `Unavailable` döner.

use serde::Serialize;
use windows::{
    core::HSTRING,
    ApplicationModel::{Activation::ActivationKind, AppInstance, StartupTask, StartupTaskState},
};

const TASK_ID: &str = "NahrokStartup";

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupState {
    Enabled,
    Disabled,
    /// Kullanıcı Windows ayarlarından kapatmış: uygulama yeniden açamaz, yalnız kullanıcı açabilir
    DisabledByUser,
    DisabledByPolicy,
    EnabledByPolicy,
    /// Paket kimliği yok (kurulu sürüm değil)
    Unavailable,
}

impl From<StartupTaskState> for StartupState {
    fn from(state: StartupTaskState) -> Self {
        match state {
            StartupTaskState::Enabled => Self::Enabled,
            StartupTaskState::DisabledByUser => Self::DisabledByUser,
            StartupTaskState::DisabledByPolicy => Self::DisabledByPolicy,
            StartupTaskState::EnabledByPolicy => Self::EnabledByPolicy,
            _ => Self::Disabled,
        }
    }
}

fn task() -> windows::core::Result<StartupTask> {
    StartupTask::GetAsync(&HSTRING::from(TASK_ID))?.join()
}

/// Bekletir (WinRT çağrısı): ana iş parçacığında değil, async komuttan çağrılır.
pub fn state() -> StartupState {
    task().and_then(|t| t.State()).map(StartupState::from).unwrap_or(StartupState::Unavailable)
}

/// Bekletir. Döner: değişiklikten sonraki durum.
pub fn set_enabled(enabled: bool) -> StartupState {
    let Ok(task) = task() else { return StartupState::Unavailable };
    let result = if enabled {
        task.RequestEnableAsync().and_then(|op| op.join()).map(StartupState::from)
    } else {
        task.Disable().and_then(|_| task.State()).map(StartupState::from)
    };
    result.unwrap_or_else(|_| state())
}

/// Uygulama bu açılışta Windows'un başlangıç görevi tarafından mı başlatıldı?
pub fn launched_at_login() -> bool {
    AppInstance::GetActivatedEventArgs()
        .and_then(|args| args.Kind())
        .map(|kind| kind == ActivationKind::StartupTask)
        .unwrap_or(false)
}
