use std::process::Command;

pub fn launch_app(path: &str, urls: &[String]) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        launch_windows(path, urls)
    }

    #[cfg(target_os = "macos")]
    {
        launch_macos(path, urls)
    }
}

#[cfg(target_os = "windows")]
fn launch_windows(path: &str, urls: &[String]) -> Result<(), String> {
    if path.is_empty() {
        return Err("No path provided".into());
    }

    if path.starts_with("shell:AppsFolder\\") {
        let aumid = &path["shell:AppsFolder\\".len()..];
        return launch_uwp(aumid);
    }

    let p = std::path::Path::new(path);
    if p.is_file()
        && p.extension()
            .map(|e| e.eq_ignore_ascii_case("exe"))
            .unwrap_or(false)
    {
        let mut cmd = Command::new(path);

        if !urls.is_empty() {
            let lower = path.to_lowercase();
            if lower.contains("firefox")
                || lower.contains("waterfox")
                || lower.contains("librewolf")
            {
                cmd.arg("-new-window");
            } else {
                cmd.arg("--new-window");
            }
        }

        for url in urls {
            cmd.arg(url);
        }
        cmd.spawn()
            .map_err(|e| format!("Failed to launch {path}: {e}"))?;
        return Ok(());
    }

    Err(format!("Don't know how to launch: {path}"))
}

#[cfg(target_os = "windows")]
fn launch_uwp(aumid: &str) -> Result<(), String> {
    use windows::{
        core::PCWSTR,
        Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED,
        },
        Win32::UI::Shell::{IApplicationActivationManager, ACTIVATEOPTIONS},
    };

    const CLSID_AAM: windows::core::GUID =
        windows::core::GUID::from_u128(0x45BA127D_10A8_46EA_8AB7_56EA9078943C);

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

        let aam: IApplicationActivationManager =
            CoCreateInstance(&CLSID_AAM, None, CLSCTX_LOCAL_SERVER)
                .map_err(|e| format!("Failed to create activation manager: {e}"))?;

        let wide: Vec<u16> = aumid.encode_utf16().chain(std::iter::once(0)).collect();

        aam.ActivateApplication(PCWSTR(wide.as_ptr()), None, ACTIVATEOPTIONS(0))
            .map_err(|e| format!("Failed to activate {aumid}: {e}"))?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_macos(path: &str, urls: &[String]) -> Result<(), String> {
    let mut cmd = Command::new("open");
    for url in urls {
        cmd.arg(url);
    }
    cmd.arg(path);
    cmd.spawn().map_err(|e| e.to_string())?;

    Ok(())
}
