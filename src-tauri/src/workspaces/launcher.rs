use std::process::Command;

pub fn launch_app(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        launch_windows(path)
    }

    #[cfg(target_os = "macos")]
    {
        launch_macos(path)
    }
}

#[cfg(target_os = "windows")]
fn launch_windows(path: &str) -> Result<(), String> {
    let p = std::path::Path::new(path);

    if p.is_file() {
        Command::new(path).spawn().map_err(|e| e.to_string())?;
    } else {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_macos(path: &str) -> Result<(), String> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}
