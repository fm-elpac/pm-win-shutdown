use std::thread;

use windows::{
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, PAINTSTRUCT, SetBkMode,
            TRANSPARENT,
        },
        System::{LibraryLoader::GetModuleHandleW, Shutdown::ShutdownBlockReasonCreate},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect,
            GetMessageW, GetSystemMetrics, HTCAPTION, IDC_ARROW, LoadCursorW, MB_OK, MSG,
            MessageBoxW, PBT_APMRESUMEAUTOMATIC, PBT_APMSUSPEND, PostQuitMessage, RegisterClassW,
            SM_CXSCREEN, WM_CLOSE, WM_DESTROY, WM_ENDSESSION, WM_NCHITTEST, WM_PAINT,
            WM_POWERBROADCAST, WM_QUERYENDSESSION, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
            WS_POPUP, WS_VISIBLE,
        },
    },
    core::{PCWSTR, w},
};

use crate::{
    dir_log::log_line,
    script::{run_script, run_shutdown},
};

pub fn mbox(text: &str, icon: windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE) {
    let text_w: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(
            Some(HWND::default()),
            PCWSTR(text_w.as_ptr()),
            w!("pm-win-shutdown"),
            MB_OK | icon,
        );
    }
}

/// 隐藏窗口的消息处理
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        // 关机/重启/注销 前, 系统询问每个顶层窗口. 返回 TRUE 表示允许.
        // 在这里同步执行 logout.bat (本程序的 核心功能)
        WM_QUERYENDSESSION => {
            // 立即启动 关机线程
            thread::spawn(|| run_shutdown());

            // 立即 阻止关机
            LRESULT(0)
        }
        WM_ENDSESSION => {
            if wparam.0 != 0 {
                log_line("system message: WM_ENDSESSION (session is ending)");
                unsafe { PostQuitMessage(0) };
            }
            LRESULT(0)
        }
        WM_POWERBROADCAST => {
            let m = wparam.0 as u32;
            match m {
                PBT_APMSUSPEND => {
                    log_line("system message: PBT_APMSUSPEND (entering sleep)");
                    // 系统无法等待 sleep_before.bat 执行完成, 就会 sleep
                    //run_script("sleep_before.bat");
                }
                // PBT_APMRESUMESUSPEND
                PBT_APMRESUMEAUTOMATIC => {
                    log_line("system message: PBT_APMRESUMEAUTOMATIC (resume from sleep)");
                    run_script("sleep_after.bat");
                }
                _ => {}
            }
            // TRUE
            LRESULT(1)
        }
        // 整个窗口都视为 "标题栏": 鼠标按住即可拖动移动位置
        WM_NCHITTEST => LRESULT(HTCAPTION as isize),

        // 绘制: 纯色小方块
        WM_PAINT => {
            unsafe {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                let mut rc = RECT::default();
                let _ = GetClientRect(hwnd, &mut rc);
                // 深青色方块 (COLORREF 为 0x00BBGGRR)
                //let brush = CreateSolidBrush(COLORREF(0x0080_6000));
                // 黑色
                let brush = CreateSolidBrush(COLORREF(0x0000_0000));
                FillRect(hdc, &rc, brush);
                let _ = DeleteObject(brush.into());
                let _ = SetBkMode(hdc, TRANSPARENT);
                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        // 无法关闭: Alt+F4 / SC_CLOSE 全部忽略 (原来这里是 DestroyWindow)
        WM_CLOSE => LRESULT(0),

        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub fn run_win() {
    unsafe {
        let hinst = HINSTANCE(
            GetModuleHandleW(None)
                .map(|h| h.0)
                .unwrap_or(std::ptr::null_mut()),
        );
        let class = w!("pm-win-shutdown-hidden-window");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinst,
            lpszClassName: class,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            ..Default::default()
        };
        if RegisterClassW(&wc) == 0 {
            log_line("RegisterClassW failed");
            return;
        }

        // 可见的小方块窗口 (WS_POPUP):
        //   - 固定大小, 无边框/标题栏/按钮/系统菜单
        //   - WS_EX_TOOLWINDOW: 不出现在任务栏和 Alt+Tab
        //   - WS_EX_NOACTIVATE: 点击时不抢焦点
        // 注意不能用 HWND_MESSAGE 消息窗口: 收不到广播消息;
        // 也不能再隐藏: 微软文档明确说, 没有可见窗口的程序
        // 对 WM_QUERYENDSESSION 返回 FALSE 无效 (会被直接终止).
        const SIZE: i32 = 32;
        //let x = GetSystemMetrics(SM_CXSCREEN) - SIZE - 24;
        //let y = GetSystemMetrics(SM_CYSCREEN) - SIZE - 64;
        // 右
        let x = GetSystemMetrics(SM_CXSCREEN) - SIZE - 256;
        // 上
        let y = 32;
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class,
            w!("pm-win-shutdown"),
            WS_POPUP | WS_VISIBLE,
            x,
            y,
            SIZE,
            SIZE,
            Some(HWND::default()),
            None,
            Some(hinst),
            None,
        );
        let hwnd = match hwnd {
            Ok(hwnd) => hwnd,
            Err(e) => {
                log_line(&format!("CreateWindowExW failed: {:?}", e));
                return;
            }
        };

        // 关机时, 系统 "阻止关机的应用" 界面会显示这段文字
        let _ = ShutdownBlockReasonCreate(hwnd, w!("正在运行 logout.bat"));

        // 消息循环
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, Some(HWND::default()), 0, 0).0 > 0 {
            DispatchMessageW(&msg);
        }
        let _ = DestroyWindow(hwnd);
    }
}
