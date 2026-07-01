#![windows_subsystem = "windows"]
#![allow(unsafe_op_in_unsafe_fn, dead_code)]

use core::ptr::null_mut;
use std::mem::{size_of, zeroed};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::Graphics::Gdi::HBRUSH;
use windows_sys::Win32::System::Threading::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

unsafe extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    fn CreateMutexW(
        lpMutexAttributes: *mut core::ffi::c_void,
        bInitialOwner: BOOL,
        lpName: *const u16,
    ) -> *mut core::ffi::c_void;
    fn CloseHandle(hObject: *mut core::ffi::c_void) -> BOOL;
    fn timeBeginPeriod(uPeriod: u32) -> u32;
    fn timeEndPeriod(uPeriod: u32) -> u32;
}

const CLASS_NAME: &str = "JitterFilterWnd";
const WM_TRAY_ICON: u32 = WM_APP + 1;
const ID_TRAY: u32 = 1;
const ID_EXIT: usize = 100;
const ID_COFFEE: usize = 101;

const INTERVAL_MS: u64 = 100;
const RATIO_THRESHOLD: i32 = 15;
const TIME_RESET_MS: u64 = 60;

#[derive(PartialEq)]
enum FilterState { Clean, Suppress }

static mut LAST_X: i32 = 0;
static mut LAST_Y: i32 = 0;
static mut LAST_T: Option<Instant> = None;
static mut STATE: FilterState = FilterState::Clean;
static mut INTV_START: Option<Instant> = None;
static mut START_X: i32 = 0;
static mut START_Y: i32 = 0;
static mut ACC_DX: i32 = 0;
static mut ACC_DY: i32 = 0;
static mut ACC_PATH: i32 = 0;
static mut SKIP: bool = false;

unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code < 0 || w_param as u32 != WM_MOUSEMOVE {
        return unsafe { CallNextHookEx(null_mut(), n_code, w_param, l_param) };
    }

    if unsafe { SKIP } {
        unsafe { SKIP = false };
        return 1;
    }

    let pt = unsafe { &*(l_param as *const MSLLHOOKSTRUCT) };
    let now = Instant::now();

    unsafe {
        match LAST_T {
            None => {
                LAST_X = pt.pt.x;
                LAST_Y = pt.pt.y;
                LAST_T = Some(now);
                STATE = FilterState::Clean;
                INTV_START = None;
                ACC_DX = 0;
                ACC_DY = 0;
                ACC_PATH = 0;
                return 1;
            }
            Some(last) => {
                let dt = now - last;

                if dt > Duration::from_millis(TIME_RESET_MS) {
                    LAST_X = pt.pt.x;
                    LAST_Y = pt.pt.y;
                    LAST_T = Some(now);
                    STATE = FilterState::Clean;
                    INTV_START = None;
                    ACC_DX = 0;
                    ACC_DY = 0;
                    ACC_PATH = 0;
                    return unsafe { CallNextHookEx(null_mut(), n_code, w_param, l_param) };
                }

                let start: *mut Option<Instant> = &raw mut INTV_START;
                if (*start).is_none() {
                    *start = Some(now);
                    START_X = pt.pt.x;
                    START_Y = pt.pt.y;
                }

                let dx = pt.pt.x - LAST_X;
                let dy = pt.pt.y - LAST_Y;

                ACC_DX += dx;
                ACC_DY += dy;
                ACC_PATH += dx.abs() + dy.abs();

                let elapsed = now - (*start).unwrap();
                if elapsed >= Duration::from_millis(INTERVAL_MS) {
                    let net = ACC_DX.abs() + ACC_DY.abs();
                    let is_jitter = net == 0 || ACC_PATH * 10 > RATIO_THRESHOLD * net;

                    if is_jitter {
                        let corr_x = START_X - pt.pt.x;
                        let corr_y = START_Y - pt.pt.y;
                        if corr_x != 0 || corr_y != 0 {
                            SKIP = true;
                            let mut input: INPUT = zeroed();
                            input.r#type = INPUT_MOUSE;
                            input.Anonymous.mi.dwFlags = MOUSEEVENTF_MOVE;
                            input.Anonymous.mi.dx = corr_x;
                            input.Anonymous.mi.dy = corr_y;
                            SendInput(1, &raw mut input, size_of::<INPUT>() as i32);
                        }
                        STATE = FilterState::Suppress;
                    } else {
                        STATE = FilterState::Clean;
                    }

                    *start = Some(now);
                    START_X = pt.pt.x;
                    START_Y = pt.pt.y;
                    ACC_DX = 0;
                    ACC_DY = 0;
                    ACC_PATH = 0;
                }

                if STATE == FilterState::Suppress {
                    return 1;
                }

                LAST_X = pt.pt.x;
                LAST_Y = pt.pt.y;
                LAST_T = Some(now);
                unsafe { CallNextHookEx(null_mut(), n_code, w_param, l_param) }
            }
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            let mut nid: NOTIFYICONDATAW = zeroed();
            nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = ID_TRAY;
            Shell_NotifyIconW(NIM_DELETE, &raw const nid);
            PostQuitMessage(0);
            return 0;
        }
        WM_TRAY_ICON => {
            if l_param as u32 == WM_RBUTTONUP {
                let hmenu = CreatePopupMenu();
                if !hmenu.is_null() {
                    let coffee = encode_wide("Buy me a coffee ☕\0");
                    let exit = encode_wide("Exit\0");
                    AppendMenuW(hmenu, MF_STRING, ID_COFFEE, coffee.as_ptr());
                    AppendMenuW(hmenu, MF_SEPARATOR, 0, null_mut());
                    AppendMenuW(hmenu, MF_STRING, ID_EXIT, exit.as_ptr());

                    let mut pt = POINT { x: 0, y: 0 };
                    GetCursorPos(&mut pt);
                    SetForegroundWindow(hwnd);
                    TrackPopupMenu(hmenu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, null_mut());
                    PostMessageW(hwnd, WM_NULL, 0, 0);
                    DestroyMenu(hmenu);
                }
            } else if l_param as u32 == WM_LBUTTONDBLCLK {
                let mut nid: NOTIFYICONDATAW = zeroed();
                nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uID = ID_TRAY;
                nid.uFlags = NIF_INFO;
                copy_wide(&mut nid.szInfoTitle, &encode_wide("Jitter Filter\0"));
                copy_wide(&mut nid.szInfo, &encode_wide("Active \u{2014} filtering trackpad jitter\0"));
                nid.dwInfoFlags = NIIF_INFO;
                Shell_NotifyIconW(NIM_MODIFY, &raw const nid);
            }
            return 0;
        }
        WM_COMMAND => {
            let id = (w_param as usize) & 0xFFFF;
            match id {
                ID_EXIT => {
                    DestroyWindow(hwnd);
                    return 0;
                }
                ID_COFFEE => {
                    ShellExecuteW(
                        hwnd,
                        encode_wide("open\0").as_ptr(),
                        encode_wide("https://buymeacoffee.com/bartosz.janiak\0").as_ptr(),
                        null_mut(),
                        null_mut(),
                        SW_SHOW,
                    );
                    return 0;
                }
                _ => {}
            }
        }
        _ => {}
    }

    DefWindowProcW(hwnd, msg, w_param, l_param)
}

fn encode_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn copy_wide(dst: &mut [u16], src: &[u16]) {
    let len = src.len().min(dst.len());
    dst[..len].copy_from_slice(&src[..len]);
}

fn main() {
    unsafe {
        let mutex_name = encode_wide("Local\\JitterFilterSingleton\0");
        let mutex = CreateMutexW(null_mut(), 0, mutex_name.as_ptr());
        if mutex.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            if !mutex.is_null() {
                CloseHandle(mutex);
            }
            return;
        }

        let instance = GetModuleHandleW(null_mut());

        let class_w: Vec<u16> = CLASS_NAME.encode_utf16().chain(std::iter::once(0)).collect();

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hCursor: null_mut(),
            hbrBackground: null_mut::<core::ffi::c_void>() as HBRUSH,
            lpszMenuName: null_mut(),
            lpszClassName: class_w.as_ptr(),
        };

        if RegisterClassW(&raw const wc) == 0 {
            return;
        }

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class_w.as_ptr(),
            class_w.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        );

        if hwnd.is_null() {
            return;
        }

        let mut icon_path = std::env::current_exe().unwrap_or_default();
        icon_path.set_file_name("jitter.ico");
        let icon_wide: Vec<u16> = icon_path.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
        let hicon = LoadImageW(
            null_mut(),
            icon_wide.as_ptr(),
            IMAGE_ICON,
            0,
            0,
            LR_LOADFROMFILE,
        );
        let hicon = if hicon.is_null() {
            LoadIconW(null_mut(), IDI_APPLICATION)
        } else {
            hicon as HICON
        };

        let mut nid: NOTIFYICONDATAW = zeroed();
        nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = ID_TRAY;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = WM_TRAY_ICON;
        nid.hIcon = hicon;
        copy_wide(&mut nid.szTip, &encode_wide("Jitter Filter\0"));

        Shell_NotifyIconW(NIM_ADD, &raw const nid);

        let running = Arc::new(AtomicBool::new(true));
        let running_hook = running.clone();

        let hook_handle = std::thread::spawn(move || unsafe {
            timeBeginPeriod(1);

            let instance = GetModuleHandleW(null_mut());
            let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(low_level_mouse_proc), instance, 0);

            if !hook.is_null() {
                SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL);

                let mut msg = MSG {
                    hwnd: null_mut(),
                    message: 0,
                    wParam: 0,
                    lParam: 0,
                    time: 0,
                    pt: POINT { x: 0, y: 0 },
                };

                while running_hook.load(Ordering::Relaxed) {
                    while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                        if msg.message == WM_QUIT {
                            running_hook.store(false, Ordering::Relaxed);
                            break;
                        }
                        TranslateMessage(&raw const msg);
                        DispatchMessageW(&raw const msg);
                    }
                    Sleep(1);
                }

                UnhookWindowsHookEx(hook);
            }

            timeEndPeriod(1);
        });

        SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL);

        let mut msg = MSG {
            hwnd: null_mut(),
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT { x: 0, y: 0 },
        };

        while GetMessageW(&mut msg, null_mut(), 0, 0) != 0 {
            TranslateMessage(&raw const msg);
            DispatchMessageW(&raw const msg);
        }

        running.store(false, Ordering::Relaxed);
        hook_handle.join().unwrap();
    }
}