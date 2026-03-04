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
    const TERMINAL_BUNDLE_IDS: &[&str] = &[
        "com.apple.Terminal",
        "com.googlecode.iterm2",
        "org.alacritty",
        "net.kovidgoyal.kitty",
        "dev.warp.Warp-Stable",
        "com.microsoft.VSCode",
        "io.tabby",
    ];

    use objc2_app_kit::NSWorkspace;
    let workspace = unsafe { NSWorkspace::sharedWorkspace() };
    if let Some(app) = unsafe { workspace.frontmostApplication() } {
        if let Some(bundle_id) = unsafe { app.bundleIdentifier() } {
            let id = bundle_id.to_string();
            return TERMINAL_BUNDLE_IDS.iter().any(|t| id == *t);
        }
    }
    false
}

#[cfg(target_os = "macos")]
pub fn activate_terminal_window() {}

#[cfg(target_os = "linux")]
pub fn is_terminal_focused() -> bool {
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

    if let Ok((conn, screen_num)) = x11rb::connect(None) {
        let screen = &conn.setup().roots[screen_num];

        if let Ok(atom_reply) =
            x11rb::protocol::xproto::intern_atom(&conn, false, b"_NET_ACTIVE_WINDOW")
        {
            if let Ok(atom) = atom_reply.reply() {
                if let Ok(prop) = x11rb::protocol::xproto::get_property(
                    &conn,
                    false,
                    screen.root,
                    atom.atom,
                    x11rb::protocol::xproto::AtomEnum::WINDOW,
                    0,
                    1,
                ) {
                    if let Ok(prop_reply) = prop.reply() {
                        if let Some(window_id) = prop_reply.value32().and_then(|mut v| v.next()) {
                            if let Ok(class_atom) = x11rb::protocol::xproto::intern_atom(
                                &conn,
                                false,
                                b"WM_CLASS",
                            ) {
                                if let Ok(class_atom_reply) = class_atom.reply() {
                                    if let Ok(class_prop) =
                                        x11rb::protocol::xproto::get_property(
                                            &conn,
                                            false,
                                            window_id,
                                            class_atom_reply.atom,
                                            x11rb::protocol::xproto::AtomEnum::STRING,
                                            0,
                                            256,
                                        )
                                    {
                                        if let Ok(class_reply) = class_prop.reply() {
                                            let class_str = String::from_utf8_lossy(
                                                &class_reply.value,
                                            )
                                            .to_lowercase();
                                            return TERMINAL_CLASSES
                                                .iter()
                                                .any(|t| class_str.contains(t));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
pub fn activate_terminal_window() {}
