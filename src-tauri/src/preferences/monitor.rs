pub fn set_brightness(percent: u8) -> Result<(), String> {
    let percent = percent.clamp(0, 100);

    #[cfg(target_os = "windows")]
    {
        set_brightness_windows(percent)
    }

    #[cfg(target_os = "macos")]
    {
        set_brightness_macos(percent)
    }
}

#[cfg(target_os = "windows")]
use std::process::Command;
use windows::{
    core::BOOL,
    Win32::{
        Foundation::{HANDLE, LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR},
    },
};

#[cfg(target_os = "windows")]
fn set_brightness_windows(percent: u8) -> Result<(), String> {
    let internal_result = set_windows_internal(percent);

    let external_result = set_windows_external(percent);

    if internal_result.is_err() && external_result.is_err() {
        return Err(format!(
            "Internal: {:?}, External: {:?}",
            internal_result.err(),
            external_result.err()
        ));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn set_windows_internal(percent: u8) -> Result<(), String> {
    let script = format!(
        "(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods).WmiSetBrightness(1,{})",
        percent
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn set_windows_external(percent: u8) -> Result<(), String> {
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(percent as isize),
        );
    }

    Ok(())
}

#[cfg(target_os = "windows")]
#[link(name = "Dxva2")]
unsafe extern "system" {
    fn GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor: HMONITOR, count: *mut u32) -> BOOL;

    fn GetPhysicalMonitorsFromHMONITOR(
        hmonitor: HMONITOR,
        count: u32,
        monitors: *mut PHYSICAL_MONITOR,
    ) -> BOOL;

    fn DestroyPhysicalMonitors(count: u32, monitors: *mut PHYSICAL_MONITOR) -> BOOL;

    fn SetVCPFeature(monitor: HANDLE, code: u8, value: u32) -> BOOL;
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct PHYSICAL_MONITOR {
    handle: HANDLE,
    description: [u16; 128],
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn monitor_enum_proc(
    monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let brightness = data.0 as u32;

    let mut count = 0;

    if !unsafe { GetNumberOfPhysicalMonitorsFromHMONITOR(monitor, &mut count) }.as_bool() {
        return BOOL(1);
    }

    let mut monitors = vec![
        PHYSICAL_MONITOR {
            handle: HANDLE::default(),
            description: [0; 128],
        };
        count as usize
    ];

    if unsafe { GetPhysicalMonitorsFromHMONITOR(monitor, count, monitors.as_mut_ptr()) }.as_bool() {
        let mut success = false;

        for m in &monitors {
            if unsafe { SetVCPFeature(m.handle, 0x10, brightness) }.as_bool() {
                success = true;
            }
        }

        if !success {
            eprintln!("No monitor accepted DDC brightness command");
        }

        unsafe {
            let _ = DestroyPhysicalMonitors(count, monitors.as_mut_ptr());
        }
    }

    BOOL(1)
}

#[cfg(target_os = "macos")]
fn set_brightness_macos(percent: u8) -> Result<(), String> {
    set_macos_internal(percent)?;
    set_macos_external(percent)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn set_macos_external(percent: u8) -> Result<(), String> {
    use ddc_hi::{Ddc, Display};

    let mut success = false;

    for mut display in Display::enumerate() {
        if display.set_vcp_feature(0x10, percent as u16).is_ok() {
            success = true;
        }
    }

    if !success {
        return Err("No DDC monitor accepted brightness command".into());
    }

    Ok(())
}

#[cfg(target_os = "macos")]
#[link(name = "DisplayServices", kind = "framework")]
unsafe extern "C" {
    fn DisplayServicesSetBrightness(display: u32, brightness: f32) -> i32;
}

#[cfg(target_os = "macos")]
fn set_macos_internal(percent: u8) -> Result<(), String> {
    use core_graphics::display::CGMainDisplayID;

    let display = unsafe { CGMainDisplayID() };

    let value = percent as f32 / 100.0;

    unsafe {
        let _ = DisplayServicesSetBrightness(display, value);
    }

    Ok(())
}
