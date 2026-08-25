use std::path::{Path, PathBuf};
use windows::core::{Interface, Result, PCWSTR};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IPersistFile,
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{
    FOLDERID_Programs, IShellLinkW, SHGetKnownFolderPath, ShellLink, KF_FLAG_CREATE,
};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Creates (or refreshes) a Start Menu shortcut for this app pointing at the currently-running
/// exe, so it's discoverable via Windows Search and relaunchable if the window is closed by
/// mistake. Runs on its own thread with its own COM apartment, independent of the main
/// Tauri/WebView2 thread. Idempotent and best-effort -- a failure here must never affect startup.
pub fn ensure_start_menu_shortcut(app_name: &'static str) {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    std::thread::spawn(move || {
        if let Err(err) = create_shortcut(&exe, app_name) {
            eprintln!("failed to create Start Menu shortcut: {err}");
        }
    });
}

fn create_shortcut(exe: &Path, app_name: &str) -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let result = create_shortcut_inner(exe, app_name);
        CoUninitialize();
        result
    }
}

unsafe fn create_shortcut_inner(exe: &Path, app_name: &str) -> Result<()> {
    let programs_ptr = SHGetKnownFolderPath(&FOLDERID_Programs, KF_FLAG_CREATE, None)?;
    let programs = programs_ptr.to_string();
    CoTaskMemFree(Some(programs_ptr.0 as _));
    // A malformed path from the OS should never happen; if it does, skip silently rather than
    // forcing an artificial error value just to satisfy the return type.
    let Ok(programs) = programs else {
        return Ok(());
    };

    let lnk_path: PathBuf = [programs.as_str(), &format!("{app_name}.lnk")].iter().collect();

    let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
    shell_link.SetPath(PCWSTR(wide(&exe.display().to_string()).as_ptr()))?;
    if let Some(dir) = exe.parent() {
        let _ = shell_link.SetWorkingDirectory(PCWSTR(wide(&dir.display().to_string()).as_ptr()));
    }
    let _ = shell_link.SetDescription(PCWSTR(wide(app_name).as_ptr()));
    let _ = shell_link.SetIconLocation(PCWSTR(wide(&exe.display().to_string()).as_ptr()), 0);

    let persist_file: IPersistFile = shell_link.cast()?;
    persist_file.Save(PCWSTR(wide(&lnk_path.display().to_string()).as_ptr()), true)
}
