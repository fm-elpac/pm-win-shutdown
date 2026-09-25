use std::fs;

use windows::{
    Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
        RegCreateKeyExW, RegSetValueExW,
    },
    core::{PCWSTR, w},
};

use crate::dir_log::{base_dir, log_line};

/// --install : 安装
pub fn install_self() -> Result<(), String> {
    let dir = base_dir();
    fs::create_dir_all(dir.join("script")).map_err(|e| e.to_string())?;

    // (1) 复制自身
    let src = std::env::current_exe().map_err(|e| e.to_string())?;
    let dst = dir.join("pm-win-shutdown.exe");
    if src != dst {
        fs::copy(&src, &dst).map_err(|e| format!("copy failed: {}", e))?;
    }
    log_line(&format!("installed to {}", dst.display()));

    // (2) 注册表: 当前用户登录后自动运行 "--run"
    //   HKCU\Software\Microsoft\Windows\CurrentVersion\Run
    //   "pm-win-shutdown" = "\"<dst>\" --run"
    let run_cmd = format!("\"{}\" --run", dst.display());
    let run_cmd_w: Vec<u16> = run_cmd.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let run_cmd_bytes: &[u8] =
            std::slice::from_raw_parts(run_cmd_w.as_ptr() as *const u8, run_cmd_w.len() * 2);

        let mut key = HKEY::default();
        let r = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            Some(0),
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut key,
            None,
        );
        if r.is_err() {
            return Err(format!("RegCreateKeyExW failed ({})", r.0));
        }

        let r = RegSetValueExW(
            key,
            w!("pm-win-shutdown"),
            Some(0),
            REG_SZ,
            Some(run_cmd_bytes),
        );
        let _ = RegCloseKey(key);
        if r.is_err() {
            return Err(format!("RegSetValueExW failed ({})", r.0));
        }
    }
    log_line(
        "registry Run entry written (HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\\pm-win-shutdown)",
    );

    Ok(())
}
