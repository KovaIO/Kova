#[cfg(target_os = "windows")]
pub fn set_brightness(percent: u8) -> Result<(), String> {
    use std::ffi::c_void;
    use windows::core::{BOOL, PCWSTR};
    use windows::Win32::Foundation::{LPARAM, RECT};
    use windows::Win32::{
        Devices::Display::{DISPLAY_BRIGHTNESS, IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS},
        Foundation::CloseHandle,
        Graphics::Gdi::{
            EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, DISPLAY_DEVICEW,
            DISPLAY_DEVICE_ACTIVE, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
        },
        Storage::FileSystem::{
            CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE,
            OPEN_EXISTING,
        },
        System::IO::DeviceIoControl,
        UI::WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME,
    };

    let percent = percent.clamp(0, 100);

    unsafe extern "system" fn enum_monitors(
        handle: HMONITOR,
        _: HDC,
        _: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        unsafe { &mut *(data.0 as *mut Vec<HMONITOR>) }.push(handle);
        BOOL(1)
    }

    let mut hmonitors = Vec::<HMONITOR>::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(enum_monitors),
            LPARAM(&mut hmonitors as *mut _ as isize),
        );
    }

    let mut internal_ok = false;

    for hmonitor in hmonitors {
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if unsafe { GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut MONITORINFO) } == BOOL(0)
        {
            continue;
        }

        for dev_num in 0.. {
            let mut dev = DISPLAY_DEVICEW {
                cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                ..Default::default()
            };
            if unsafe {
                EnumDisplayDevicesW(
                    PCWSTR(info.szDevice.as_ptr()),
                    dev_num,
                    &mut dev,
                    EDD_GET_DEVICE_INTERFACE_NAME,
                )
            } == BOOL(0)
            {
                break;
            }

            if dev.StateFlags.0 & DISPLAY_DEVICE_ACTIVE.0 == 0 {
                continue;
            }

            let Ok(handle) = (unsafe {
                CreateFileW(
                    PCWSTR(dev.DeviceID.as_ptr()),
                    (FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0) as u32,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    None,
                    OPEN_EXISTING,
                    Default::default(),
                    None,
                )
            }) else {
                continue;
            };

            let mut brightness = DISPLAY_BRIGHTNESS {
                ucACBrightness: percent,
                ucDCBrightness: percent,
                ucDisplayPolicy: 3,
            };
            let mut bytes = 0u32;
            let ok = unsafe {
                DeviceIoControl(
                    handle,
                    IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS,
                    Some(&mut brightness as *mut _ as *mut c_void),
                    std::mem::size_of::<DISPLAY_BRIGHTNESS>() as u32,
                    None,
                    0,
                    Some(&mut bytes),
                    None,
                )
            };
            unsafe {
                let _ = CloseHandle(handle);
            }

            if ok.is_ok() {
                internal_ok = true;
                break;
            }
        }
    }

    if internal_ok || set_windows_external(percent) {
        Ok(())
    } else {
        Err("No monitor accepted brightness command".into())
    }
}

#[cfg(target_os = "windows")]
fn set_windows_external(percent: u8) -> bool {
    use windows::core::BOOL;
    use windows::Win32::Foundation::{HANDLE, LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};

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

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct PHYSICAL_MONITOR {
        handle: HANDLE,
        description: [u16; 128],
    }

    unsafe extern "system" fn enum_proc(
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
                handle: Default::default(),
                description: [0; 128]
            };
            count as usize
        ];

        if unsafe { GetPhysicalMonitorsFromHMONITOR(monitor, count, monitors.as_mut_ptr()) }
            .as_bool()
        {
            for m in &monitors {
                unsafe {
                    let _ = SetVCPFeature(m.handle, 0x10, brightness);
                };
            }
            unsafe {
                let _ = DestroyPhysicalMonitors(count, monitors.as_mut_ptr());
            }
        }

        BOOL(1)
    }

    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(enum_proc), LPARAM(percent as isize));
    }

    true
}

#[cfg(target_os = "macos")]
pub fn set_brightness(percent: u8) -> Result<(), String> {
    use core_graphics::display::CGMainDisplayID;

    let percent = percent.clamp(0, 100);
    let value = percent as f32 / 100.0;

    #[link(name = "DisplayServices", kind = "framework")]
    extern "C" {
        fn DisplayServicesSetBrightness(display: u32, brightness: f32) -> i32;
    }

    unsafe {
        let _ = DisplayServicesSetBrightness(CGMainDisplayID(), value);
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn get_brightness() -> Result<u8, String> {
    use windows::core::BOOL;
    use windows::Win32::Foundation::{HANDLE, LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};

    #[link(name = "Dxva2")]
    unsafe extern "system" {
        fn GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor: HMONITOR, count: *mut u32) -> BOOL;
        fn GetPhysicalMonitorsFromHMONITOR(
            hmonitor: HMONITOR,
            count: u32,
            monitors: *mut PHYSICAL_MONITOR,
        ) -> BOOL;
        fn DestroyPhysicalMonitors(count: u32, monitors: *mut PHYSICAL_MONITOR) -> BOOL;
        fn GetVCPFeatureAndVCPFeatureReply(
            monitor: HANDLE,
            code: u8,
            pvct: *mut VCP_CODE_TYPE,
            current_value: *mut u32,
            maximum_value: *mut u32,
        ) -> BOOL;
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct PHYSICAL_MONITOR {
        handle: HANDLE,
        description: [u16; 128],
    }

    #[repr(C)]
    struct VCP_CODE_TYPE {
        _unused: [u8; 0],
    }

    struct BrightnessResult {
        value: Option<u8>,
    }

    let mut result = BrightnessResult { value: None };

    unsafe extern "system" fn enum_proc(
        monitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        let result = &mut *(data.0 as *mut BrightnessResult);
        let mut count = 0;

        if !unsafe { GetNumberOfPhysicalMonitorsFromHMONITOR(monitor, &mut count) }.as_bool() {
            return BOOL(1);
        }

        let mut monitors = vec![
            PHYSICAL_MONITOR {
                handle: Default::default(),
                description: [0; 128],
            };
            count as usize
        ];

        if unsafe { GetPhysicalMonitorsFromHMONITOR(monitor, count, monitors.as_mut_ptr()) }
            .as_bool()
        {
            for m in &monitors {
                let mut current = 0u32;
                let mut maximum = 0u32;
                if unsafe {
                    GetVCPFeatureAndVCPFeatureReply(
                        m.handle,
                        0x10,
                        std::ptr::null_mut(),
                        &mut current,
                        &mut maximum,
                    )
                }
                .as_bool()
                    && maximum > 0
                {
                    result.value = Some(((current as f64 / maximum as f64) * 100.0) as u8);
                    break;
                }
            }
            unsafe {
                let _ = DestroyPhysicalMonitors(count, monitors.as_mut_ptr());
            }
        }

        if result.value.is_some() {
            BOOL(0)
        } else {
            BOOL(1)
        }
    }

    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(enum_proc),
            LPARAM(&mut result as *mut _ as isize),
        );
    }

    result
        .value
        .ok_or_else(|| "No monitor reported brightness".into())
}

#[cfg(target_os = "macos")]
pub fn get_brightness() -> Result<u8, String> {
    use core_graphics::display::CGMainDisplayID;

    #[link(name = "DisplayServices", kind = "framework")]
    extern "C" {
        fn DisplayServicesGetBrightness(display: u32, brightness: *mut f32) -> i32;
    }

    let mut value: f32 = 0.0;
    unsafe {
        let _ = DisplayServicesGetBrightness(CGMainDisplayID(), &mut value);
    }

    Ok((value * 100.0).round() as u8)
}
