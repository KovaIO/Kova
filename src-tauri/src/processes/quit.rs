use sysinfo::{Pid, ProcessesToUpdate, System};

#[cfg(target_os = "windows")]
pub fn quit_process(pid: u32) -> Result<(), String> {
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE,
    };

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
        let mut window_pid = 0u32;

        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut window_pid));
        }

        let target_pid = lparam.0 as u32;

        if window_pid == target_pid {
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }

        windows::core::BOOL(1)
    }

    unsafe {
        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(pid as isize));
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn quit_process(pid: u32) -> Result<(), String> {
    use sysinfo::Signal;

    let mut sys = System::new_all();

    sys.refresh_processes(ProcessesToUpdate::All, true);

    let proc = sys.process(Pid::from_u32(pid)).ok_or("Process not found")?;

    proc.kill_with(Signal::Term)
        .ok_or("Failed to terminate process")?;

    Ok(())
}

pub fn force_quit_process(pid: u32) -> Result<(), String> {
    let mut sys = System::new_all();

    sys.refresh_processes(ProcessesToUpdate::All, true);

    let proc = sys.process(Pid::from_u32(pid)).ok_or("Process not found")?;

    proc.kill();

    Ok(())
}
