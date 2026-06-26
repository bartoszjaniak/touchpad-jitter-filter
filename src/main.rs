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

unsafe extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    fn CreateMutexW(
        lpMutexAttributes: *mut core::ffi::c_void,
        bInitialOwner: BOOL,
        lpName: *const u16,
    ) -> *mut core::ffi::c_void;
    fn CloseHandle(hObject: *mut core::ffi::c_void) -> BOOL;
}

const CLASS_NAME: &str = "JitterFilterWnd";
const WM_TRAY_ICON: u32 = WM_APP + 1;
const ID_TRAY: u32 = 1;
const ID_EXIT: usize = 100;
const ID_COFFEE: usize = 101;

const THRESHOLD: i32 = 12;
const AMPLIFY: i32 = 2;
const MIN_DIR_CHANGES: u32 = 3;

static mut LAST_X: i32 = 0;
static mut LAST_Y: i32 = 0;
static mut LAST_T: Option<Instant> = None;
static mut LAST_DX: i32 = 0;
static mut LAST_DY: i32 = 0;
static mut ACC_X: i32 = 0;
static mut ACC_Y: i32 = 0;
static mut IN_JITTER: bool = false;
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
                LAST_DX = 0;
                LAST_DY = 0;
                ACC_X = 0;
                ACC_Y = 0;
                IN_JITTER = false;
                return 1;
            }
            Some(last) => {
                let dt = now - last;

                if dt > Duration::from_millis(25) {
                    ACC_X = 0;
                    ACC_Y = 0;
                    IN_JITTER = false;
                    LAST_DX = 0;
                    LAST_DY = 0;
                    LAST_X = pt.pt.x;
                    LAST_Y = pt.pt.y;
                    LAST_T = Some(now);
                    return unsafe { CallNextHookEx(null_mut(), n_code, w_param, l_param) };
                }

                let dx = pt.pt.x - LAST_X;
                let dy = pt.pt.y - LAST_Y;
                let is_small = dx.abs() <= THRESHOLD && dy.abs() <= THRESHOLD;

                if is_small {
                    let dir_changed = (dx != 0 && LAST_DX != 0 && dx.signum() != LAST_DX.signum())
                        || (dy != 0 && LAST_DY != 0 && dy.signum() != LAST_DY.signum());

                    LAST_DX = dx;
                    LAST_DY = dy;
                    LAST_T = Some(now);

                    if dir_changed {
                        ACC_X += dx;
                        ACC_Y += dy;
                        IN_JITTER = true;
                        return 1;
                    }

                    if IN_JITTER {
                        let total_x = ACC_X + dx * AMPLIFY;
                        let total_y = ACC_Y + dy * AMPLIFY;

                        SKIP = true;
                        let mut input: INPUT = zeroed();
                        input.r#type = INPUT_MOUSE;
                        input.Anonymous.mi.dwFlags = MOUSEEVENTF_MOVE;
                        input.Anonymous.mi.dx = total_x;
                        input.Anonymous.mi.dy = total_y;
                        SendInput(1, &raw mut input, size_of::<INPUT>() as i32);

                        ACC_X = 0;
                        ACC_Y = 0;
                        IN_JITTER = false;
                        return 1;
                    }

                    LAST_X = pt.pt.x;
                    LAST_Y = pt.pt.y;
                    return unsafe { CallNextHookEx(null_mut(), n_code, w_param, l_param) };
                }

                ACC_X = 0;
                ACC_Y = 0;
                IN_JITTER = false;
                LAST_DX = 0;
                LAST_DY = 0;
                LAST_X = pt.pt.x;
                LAST_Y = pt.pt.y;
                LAST_T = Some(now);
            }
        }
    }

    unsafe { CallNextHookEx(null_mut(), n_code, w_param, l_param) }
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

        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(low_level_mouse_proc), instance, 0);

        if hook.is_null() {
            let mut nid2: NOTIFYICONDATAW = zeroed();
            nid2.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            nid2.hWnd = hwnd;
            nid2.uID = ID_TRAY;
            Shell_NotifyIconW(NIM_DELETE, &raw const nid2);
            DestroyWindow(hwnd);
            return;
        }

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

        UnhookWindowsHookEx(hook);
    }
}