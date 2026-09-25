use std::process;

use windows::{
    Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            CREATE_NO_WINDOW, CreateProcessW, GetExitCodeProcess, INFINITE, PROCESS_INFORMATION,
            STARTUPINFOW, WaitForSingleObject,
        },
    },
    core::{PCWSTR, PWSTR},
};

use crate::dir_log::{base_dir, log_line};

/// 同步执行 .bat 脚本 (阻塞直到结束)
pub fn run_script(name: &str) {
    let path = base_dir().join("script").join(name);
    if !path.is_file() {
        // 脚本不存在: 忽略, 但写入日志
        log_line(&format!("script skipped (not found): {}", name));
        return;
    }
    log_line(&format!("script start: {}", name));

    // 基础命令
    let cmd = format!("cmd.exe /C \"{}\"", path.display());

    let mut cmd_w: Vec<u16> = cmd.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut si = STARTUPINFOW::default();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut pi = PROCESS_INFORMATION::default();

        let ok = CreateProcessW(
            PCWSTR::null(),
            // 注意: lpCommandLine 必须是可写缓冲区
            Some(PWSTR(cmd_w.as_mut_ptr())),
            None,
            None,
            false,
            //
            CREATE_NO_WINDOW,
            //DETACHED_PROCESS,
            //
            None,
            PCWSTR::null(),
            &si,
            &mut pi,
        );

        match ok {
            Ok(()) => {
                // 同步等待 .bat 结束
                let _ = WaitForSingleObject(pi.hProcess, INFINITE);
                let mut code: u32 = 0;
                let _ = GetExitCodeProcess(pi.hProcess, &mut code);
                let _ = CloseHandle(pi.hProcess);
                let _ = CloseHandle(pi.hThread);
                log_line(&format!("script done: {} (exit code {})", name, code));
            }
            Err(e) => {
                log_line(&format!("script failed to start: {} ({:?})", name, e));
            }
        }
    }
}

/// 关机线程, 关机时, 异步执行 关闭脚本
pub fn run_shutdown() {
    // 写 日志 移动到此处, 从而提高 阻止关机 响应速度
    log_line("system message: WM_QUERYENDSESSION (shutdown/restart/logoff) (run_shutdown)");

    // 此处无法 运行 logout.bat (报错)
    //run_script("logout.bat");

    log_line("run_shutdown exit 0");
    // 立即退出
    process::exit(0);
}
