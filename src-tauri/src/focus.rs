/// Platform-specific focus detection.
/// Returns true if a known terminal application is the foreground window.

#[cfg(target_os = "windows")]
mod win {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
        PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetForegroundWindow, GetWindowThreadProcessId,
        IsWindowVisible, SetForegroundWindow,
    };
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};

    const TERMINAL_NAMES: &[&str] = &[
        "windowsterminal",
        "mintty",
        "conhost",
        "cmd",
        "powershell",
        "pwsh",
        "alacritty",
        "wezterm-gui",
        "hyper",
        "tabby",
        "warp",
    ];

    fn process_name_from_pid(target_pid: u32) -> Option<String> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
            let mut entry = PROCESSENTRY32W::default();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    if entry.th32ProcessID == target_pid {
                        let name = String::from_utf16_lossy(
                            &entry.szExeFile[..entry
                                .szExeFile
                                .iter()
                                .position(|&c| c == 0)
                                .unwrap_or(entry.szExeFile.len())],
                        );
                        let _ = CloseHandle(snapshot);
                        return Some(name.to_lowercase().replace(".exe", ""));
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);
            None
        }
    }

    pub fn is_terminal_focused() -> bool {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return false;
            }

            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));
            if process_id == 0 {
                return false;
            }

            if let Some(name) = process_name_from_pid(process_id) {
                return TERMINAL_NAMES.iter().any(|t| name.contains(t));
            }
            false
        }
    }

    pub fn activate_terminal_window() {
        use std::sync::atomic::{AtomicIsize, Ordering};

        static FOUND_HWND: AtomicIsize = AtomicIsize::new(0);

        unsafe extern "system" fn enum_callback(hwnd: HWND, _: LPARAM) -> BOOL {
            unsafe {
                if !IsWindowVisible(hwnd).as_bool() {
                    return BOOL(1);
                }

                let mut pid: u32 = 0;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                if pid == 0 {
                    return BOOL(1);
                }

                if let Some(name) = super::win::process_name_from_pid(pid) {
                    if name.contains("windowsterminal") || name.contains("mintty") {
                        FOUND_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
                        return BOOL(0);
                    }
                }
                BOOL(1)
            }
        }

        unsafe {
            FOUND_HWND.store(0, Ordering::SeqCst);
            let _ = EnumWindows(Some(enum_callback), LPARAM(0));
            let hwnd_val = FOUND_HWND.load(Ordering::SeqCst);
            if hwnd_val != 0 {
                let hwnd = HWND(hwnd_val as *mut _);
                let _ = SetForegroundWindow(hwnd);
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub use win::{activate_terminal_window, is_terminal_focused};

#[cfg(target_os = "macos")]
pub fn is_terminal_focused() -> bool {
    use std::process::Command;
    // Use osascript to get frontmost app bundle ID — avoids objc2 API version issues
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to get bundle identifier of first process whose frontmost is true")
        .output();

    if let Ok(output) = output {
        let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
        const TERMINAL_BUNDLE_IDS: &[&str] = &[
            "com.apple.terminal",
            "com.googlecode.iterm2",
            "org.alacritty",
            "net.kovidgoyal.kitty",
            "dev.warp.warp-stable",
            "com.microsoft.vscode",
            "io.tabby",
        ];
        return TERMINAL_BUNDLE_IDS.iter().any(|t| bundle_id.contains(t));
    }
    false
}

#[cfg(target_os = "macos")]
pub fn activate_terminal_window() {}

#[cfg(target_os = "linux")]
pub fn is_terminal_focused() -> bool {
    use std::process::Command;
    // Use xdotool to get active window class — avoids x11rb API version issues
    let output = Command::new("xdotool")
        .args(["getactivewindow", "getwindowclassname"])
        .output();

    if let Ok(output) = output {
        let class = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
        const TERMINAL_CLASSES: &[&str] = &[
            "gnome-terminal",
            "konsole",
            "xterm",
            "alacritty",
            "kitty",
            "terminator",
            "tilix",
            "xfce4-terminal",
            "code",
            "wezterm",
        ];
        return TERMINAL_CLASSES.iter().any(|t| class.contains(t));
    }
    false
}

#[cfg(target_os = "linux")]
pub fn activate_terminal_window() {}
