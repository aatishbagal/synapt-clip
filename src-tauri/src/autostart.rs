use std::path::PathBuf;

fn service_file_path() -> Result<PathBuf, String> {
    dirs::config_dir()
        .ok_or_else(|| "Could not resolve config directory".to_string())
        .map(|d| d.join("systemd").join("user").join("synaptclip.service"))
}

fn service_file_content(executable_path: &str) -> String {
    format!(
        "[Unit]\nDescription=SynaptClip clipboard manager\nAfter=graphical-session.target\n\n[Service]\nType=simple\nExecStart={executable_path}\nRestart=on-failure\nRestartSec=5\n\n[Install]\nWantedBy=default.target\n"
    )
}

pub fn install_service() -> Result<bool, String> {
    let exe = std::env::current_exe()
        .map_err(|e| e.to_string())?;
    let exe_path = exe.to_string_lossy().to_string();

    let path = service_file_path()?;

    if path.exists() {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    std::fs::write(&path, service_file_content(&exe_path)).map_err(|e| e.to_string())?;
    Ok(true)
}

pub fn is_service_installed() -> bool {
    service_file_path().map(|p| p.exists()).unwrap_or(false)
}
