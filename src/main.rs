//! pm-win-shutdown
//!
//! 静默后台程序: 监听系统 关机/注销/睡眠 消息, 并在关键时刻同步执行用户 .bat 脚本.
//!
//! 用法:
//!   pm-win-shutdown.exe --install   安装 (复制自身 + 注册表开机自启 + MessageBox 提示)
//!   pm-win-shutdown.exe --run       主功能 (登录后由注册表自启项拉起)
//!
//! 目录结构 (base = %LOCALAPPDATA%\pm-win-shutdown):
//!   pm-win-shutdown.exe             安装后的本体
//!   log\YYYYMMDD.txt                日志 (按当前日期命名, UTF-8, 仅追加)
//!   script\login.bat                登录后执行
//!   script\logout.bat               注销/关机/重启 前执行 (核心功能)
//!   script\sleep_before.bat         睡眠前执行
//!   script\sleep_after.bat          唤醒后执行

// 无控制台窗口, 静默后台运行
#![windows_subsystem = "windows"]

use windows::Win32::{
    System::Threading::SetProcessShutdownParameters,
    UI::WindowsAndMessaging::{MB_ICONERROR, MB_ICONINFORMATION},
};

mod dir_log;
mod install;
mod script;
mod ui;

use dir_log::log_line;
use install::install_self;
use script::run_script;
use ui::{mbox, run_win};

/// 关机优先级: 让本进程最先收到关机/注销询问 (WM_QUERYENDSESSION)
fn set_shutdown_priority() {
    // dwLevel 取值 0x000 ~ 0x3FF, 数值越大, 系统关闭该进程 (发询问消息) 越早.
    // 普通应用默认 0x280. 设为最高的 0x3FF, 保证本程序最先处理关机, 先跑完 logout.bat.
    const HIGHEST: u32 = 0x3FF;
    let r = unsafe { SetProcessShutdownParameters(HIGHEST, 0) };
    match r {
        Ok(()) => log_line("shutdown priority set to 0x3FF (highest, shut down first)"),
        Err(e) => log_line(&format!("SetProcessShutdownParameters failed: {:?}", e)),
    }
}

/// --run : 主功能
fn run() {
    log_line("pm-win-shutdown started (--run)");

    // 最先收到关机询问, 从而最先执行 logout.bat
    set_shutdown_priority();

    // 用户登录成功 (本程序由登录自启项拉起), 执行 login.bat
    run_script("login.bat");

    run_win();

    log_line("pm-win-shutdown exiting");
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("--install") => match install_self() {
            Ok(()) => {
                log_line("install completed");
                mbox("pm-win-shutdown 安装完成.", MB_ICONINFORMATION);
            }
            Err(e) => {
                log_line(&format!("install failed: {}", e));
                mbox(&format!("安装失败:\n{}", e), MB_ICONERROR);
            }
        },
        Some("--run") => run(),
        _ => {
            mbox(
                "用法:\n  pm-win-shutdown.exe --install   安装\n  pm-win-shutdown.exe --run       运行",
                MB_ICONINFORMATION,
            );
        }
    }
}
