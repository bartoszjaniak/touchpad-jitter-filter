#![windows_subsystem = "windows"]
#![allow(unsafe_op_in_unsafe_fn, dead_code)]

use core::ptr::null_mut;
use std::mem::{size_of, zeroed};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::Graphics::Gdi::HBRUSH;

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
const AMPLIFY: i32 = 4;

static mut LAST_PASSED_X: i32 = 0;
static mut LAST_PASSED_Y: i32 = 0;
static mut LAST_EVENT_T: Option<Instant> = None;
static mut JITTERING: bool = false;

unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code < 0 || w_param as u32 != WM_MOUSEMOVE {
        return unsafe { CallNextHookEx(null_mut(), n_code, w_param, l_param) };
    }

    let pt = unsafe { &*(l_param as *const MSLLHOOKSTRUCT) };
    let now = Instant::now();

    unsafe {
        match LAST_EVENT_T {
            None => {
                LAST_PASSED_X = pt.pt.x;
                LAST_PASSED_Y = pt.pt.y;
                LAST_EVENT_T = Some(now);
                JITTERING = false;
                return 1;
            }
            Some(last) => {
                let dx = (pt.pt.x - LAST_PASSED_X).abs();
                let dy = (pt.pt.y - LAST_PASSED_Y).abs();
                let dt = now - last;

                let within_threshold = dx <= THRESHOLD && dy <= THRESHOLD;

                if dt <= Duration::from_millis(25) {
                    if within_threshold {
                        JITTERING = true;
                        LAST_EVENT_T = Some(now);
                        return 1;
                    }
                } else {
                    JITTERING = false;
                }

                if JITTERING {
                    let delta_x = (pt.pt.x - LAST_PASSED_X) * AMPLIFY;
                    let delta_y = (pt.pt.y - LAST_PASSED_Y) * AMPLIFY;
                    let new_x = LAST_PASSED_X + delta_x;
                    let new_y = LAST_PASSED_Y + delta_y;
                    SetCursorPos(new_x, new_y);
                    LAST_PASSED_X = new_x;
                    LAST_PASSED_Y = new_y;
                    LAST_EVENT_T = Some(now);
                    return 1;
                }

                LAST_PASSED_X = pt.pt.x;
                LAST_PASSED_Y = pt.pt.y;
                LAST_EVENT_T = Some(now);
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
                copy_wide(&mut nid.szInfo, &encode_wide("Active — filtering trackpad jitter\0"));
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
            hIcon: LoadIconW(null_mut(), 32512 as *const u16),
            hCursor: null_mut(),
            hbrBackground: null_mut::<core::ffi::c_void>() as HBRUSH,
            lpszMenuName: null_mut(),
            lpszClassName: class_w.as_ptr(),
        };

        if RegisterClassW(&raw const wc) == 0 {
            return;
        }

        let hwnd = CreateWindowExW(
            0,
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

        let mut nid: NOTIFYICONDATAW = zeroed();
        nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = ID_TRAY;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = WM_TRAY_ICON;
        nid.hIcon = LoadIconW(null_mut(), 32512 as *const u16);
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