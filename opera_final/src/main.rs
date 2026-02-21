#![windows_subsystem = "windows"]
use std::{fs, process::Command, iter::once, thread, time::Duration};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{VK_RETURN, SetFocus};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::Graphics::Gdi::{CreateSolidBrush, SetTextColor, SetBkColor, HDC};
use windows_sys::Win32::Graphics::Dwm::*;

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    r as u32 | ((g as u32) << 8) | ((b as u32) << 16)
}

fn to_wstring(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(once(0)).collect()
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0); }
            0
        }
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as HDC;
            unsafe {
                SetTextColor(hdc, rgb(255, 255, 255));
                SetBkColor(hdc, rgb(60, 60, 60));
                let h_brush = CreateSolidBrush(rgb(60, 60, 60));
                h_brush as LRESULT
            }
        }
        WM_CTLCOLOREDIT => {
            let hdc = wparam as HDC;
            unsafe {
                SetTextColor(hdc, rgb(255, 255, 255));
                SetBkColor(hdc, rgb(45, 45, 45));
                let h_brush = CreateSolidBrush(rgb(45, 45, 45));
                h_brush as LRESULT
            }
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn main() {
    let h_instance = unsafe { GetModuleHandleW(std::ptr::null()) };
    unsafe {
        let class_name = to_wstring("MyClass");
        let dark_brush = CreateSolidBrush(rgb(60, 60, 60));

        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.lpfnWndProc = Some(window_proc);
        wc.hInstance = h_instance;
        wc.lpszClassName = class_name.as_ptr();
        wc.hCursor = LoadCursorW(0, IDC_ARROW as *const u16);
        wc.hbrBackground = dark_brush;
        RegisterClassW(&wc);

        let w = 250;
        let h = 160;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - w) / 2;
        let y = (screen_h - h) / 2;

        // Típusbiztos stílus összerakás (explicit u32 konverziókkal)
        let main_style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE;
        
        let hwnd = CreateWindowExW(
            0, class_name.as_ptr(), to_wstring("").as_ptr(), 
            main_style, 
            x, y, w, h, 0, 0, h_instance, std::ptr::null()
        );

        let dark_mode: i32 = 1;
        DwmSetWindowAttribute(hwnd, 20, &dark_mode as *const i32 as *const _, 4);
        
        CreateWindowExW(
            0, to_wstring("Static").as_ptr(), to_wstring("Applock:").as_ptr(), 
            WS_VISIBLE | WS_CHILD, 20, 20, 200, 20, hwnd, 0, h_instance, std::ptr::null()
        );
        
        // Itt volt a hiba: az ES_PASSWORD-öt és a WS-stílusokat u32-re kényszerítjük
        let edit_style = WS_VISIBLE | WS_CHILD | WS_BORDER | ES_PASSWORD as u32;

        let h_edit = CreateWindowExW(
            0,
            to_wstring("Edit").as_ptr(), std::ptr::null(), 
            edit_style, 
            20, 50, 180, 25, hwnd, 2, h_instance, std::ptr::null()
        );
        
        SetFocus(h_edit);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            if msg.message == WM_KEYDOWN && msg.wParam == VK_RETURN as usize {
                let mut buffer = [0u16; 32];
                let len = GetWindowTextW(h_edit, buffer.as_mut_ptr(), 32);
                if String::from_utf16_lossy(&buffer[..len as usize]).trim() == "qqq" {
                    DestroyWindow(hwnd);
                    run_logic();
                    return;
                }
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn run_logic() {
    let f = "g:\\1AZEDDIG\\MaxthonPortable\\Other\\Settings\\mappa\\opera.portable\\";
    let old = format!("{}data.pb", f);
    let exe = format!("{}opera.exe", f);
    if fs::rename(&old, &exe).is_ok() {
        let _ = Command::new(&exe).current_dir(f).status();
        let _ = fs::rename(&exe, &old);
        thread::sleep(Duration::from_secs(2));
        if let Ok(entries) = fs::read_dir(f) {
            for entry in entries.flatten() {
                if let Ok(n) = entry.file_name().into_string() {
                    if n.starts_with("scoped_dir") { let _ = fs::remove_dir_all(entry.path()); }
                }
            }
        }
    }
}
