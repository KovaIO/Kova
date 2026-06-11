use crate::workspaces::WorkspaceApp;

#[derive(Clone)]
pub struct MatchedWindow {
    pub handle: usize,
    #[allow(unused)]
    pub title: String,
}

#[cfg(target_os = "macos")]
pub fn find_all_windows(app: &WorkspaceApp) -> Vec<MatchedWindow> {
    let count = count_macos_app_instances(&app.name);
    (0..count)
        .map(|_| MatchedWindow {
            handle: 0,
            title: app.name.clone(),
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn count_macos_app_instances(app_name: &str) -> usize {
    let output = std::process::Command::new("pgrep")
        .arg("-fi")
        .arg(app_name)
        .output();

    match output {
        Ok(o) => {
            if o.status.success() && !o.stdout.is_empty() {
                // Count lines in pgrep output (each line is a PID = one instance)
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| !l.is_empty())
                    .count()
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}

#[cfg(target_os = "windows")]
pub fn find_all_windows(app: &WorkspaceApp) -> Vec<MatchedWindow> {
    find_all_windows_windows(app)
}

#[cfg(target_os = "windows")]
fn find_all_windows_windows(app: &WorkspaceApp) -> Vec<MatchedWindow> {
    use windows::{
        core::BOOL,
        Win32::{
            Foundation::{HWND, LPARAM},
            UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, IsWindowVisible},
        },
    };

    let target_exe: Option<String> =
        app.exe_path
            .as_deref()
            .map(|s| s.to_lowercase())
            .or_else(|| {
                let p = app.path.to_lowercase();
                if p.ends_with(".exe") {
                    Some(p)
                } else {
                    None
                }
            });

    let target_is_explorer = app.name.eq_ignore_ascii_case("file explorer")
        || app.name.eq_ignore_ascii_case("windows explorer")
        || target_exe
            .as_deref()
            .map(|p| p.ends_with(r"windows\explorer.exe"))
            .unwrap_or(false);

    let app_name_lower = app.name.to_lowercase();

    struct SearchData {
        target_exe: Option<String>,
        target_is_explorer: bool,
        app_name_lower: String,
        results: Vec<MatchedWindow>,
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut SearchData);

        if !IsWindowVisible(hwnd).as_bool() {
            return true.into();
        }

        let title = get_window_title(hwnd);
        if title.is_empty() {
            return true.into();
        }

        if data.target_is_explorer {
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return true.into();
            }
            let Some(exe) = process_exe(pid) else {
                return true.into();
            };
            if !exe.to_lowercase().ends_with(r"windows\explorer.exe") {
                return true.into();
            }
            let class = get_window_class(hwnd);
            if class == "CabinetWClass" || class == "ExploreWClass" {
                data.results.push(MatchedWindow {
                    handle: hwnd.0 as usize,
                    title,
                });
            }
            return true.into();
        }

        if let Some(ref target) = data.target_exe {
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return true.into();
            }

            if let Some(exe) = process_exe(pid) {
                let exe_lower = exe.to_lowercase();

                if exe_lower == *target {
                    data.results.push(MatchedWindow {
                        handle: hwnd.0 as usize,
                        title,
                    });
                    return true.into();
                }

                if let Some(parent_exe) = parent_process_exe(pid) {
                    if parent_exe.to_lowercase() == *target {
                        data.results.push(MatchedWindow {
                            handle: hwnd.0 as usize,
                            title,
                        });
                        return true.into();
                    }
                }
            }
        }

        if title.to_lowercase().contains(&data.app_name_lower) {
            data.results.push(MatchedWindow {
                handle: hwnd.0 as usize,
                title,
            });
        }

        true.into()
    }

    let mut data = SearchData {
        target_exe,
        target_is_explorer,
        app_name_lower,
        results: Vec::new(),
    };

    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut data as *mut _ as isize));
    }

    data.results
}

#[cfg(target_os = "windows")]
fn parent_process_exe(pid: u32) -> Option<String> {
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut parent_pid = None;
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                if entry.th32ProcessID == pid {
                    parent_pid = Some(entry.th32ParentProcessID);
                    break;
                }
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = windows::Win32::Foundation::CloseHandle(snap);
        process_exe(parent_pid?)
    }
}

#[cfg(target_os = "windows")]
unsafe fn get_window_title(hwnd: windows::Win32::Foundation::HWND) -> String {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW};
    let len = GetWindowTextLengthW(hwnd);
    if len == 0 {
        return String::new();
    }
    let mut buf = vec![0u16; len as usize + 1];
    GetWindowTextW(hwnd, &mut buf);
    String::from_utf16_lossy(&buf)
        .trim_matches('\0')
        .to_string()
}

#[cfg(target_os = "windows")]
unsafe fn get_window_class(hwnd: windows::Win32::Foundation::HWND) -> String {
    use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;
    let mut buf = [0u16; 256];
    let len = GetClassNameW(hwnd, &mut buf) as usize;
    String::from_utf16_lossy(&buf[..len])
}

#[cfg(target_os = "windows")]
fn process_exe(pid: u32) -> Option<String> {
    use windows::Win32::{
        Foundation::CloseHandle,
        System::{
            ProcessStatus::K32GetModuleFileNameExW,
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        },
    };
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).ok()?;
        let mut buf = vec![0u16; 32768];
        let len = K32GetModuleFileNameExW(Some(handle), None, &mut buf) as usize;
        let _ = CloseHandle(handle);
        if len == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..len]))
    }
}
