use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Com::*,
    Win32::UI::Accessibility::*,
    Win32::UI::WindowsAndMessaging::*,
};

fn get_active_window_title() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut buffer = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buffer);
        if len > 0 {
            String::from_utf16_lossy(&buffer[..len as usize])
        } else {
            "Ismeretlen".to_string()
        }
    }
}

unsafe fn variant_to_string(var: &VARIANT) -> Option<String> {
    if var.Anonymous.Anonymous.vt as u32 == VT_BSTR.0 {
        let bstr = var.Anonymous.Anonymous.Anonymous.bstrVal;
        if !bstr.is_null() {
            let len = SysStringLen(bstr) as usize;
            let slice = std::slice::from_raw_parts(bstr, len);
            return Some(String::from_utf16_lossy(slice));
        }
    }
    None
}

fn get_browser_url() -> Option<String> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;

        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;

        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return None;
        }

        let element = automation.ElementFromHandle(hwnd).ok()?;

        let title = get_active_window_title().to_lowercase();

        let is_chrome_like = title.contains("chrome")
            || title.contains("edge")
            || title.contains("brave")
            || title.contains("opera")
            || title.contains("vivaldi");

        let is_firefox = title.contains("firefox");

        if is_chrome_like {
            let cond = automation
                .CreatePropertyCondition(UIA_NamePropertyId, "Address and search bar".into())
                .ok()?;

            let urlbar = element.FindFirst(TreeScope_Subtree, &cond).ok()?;

            let mut var = VARIANT::default();
            urlbar.GetCurrentPropertyValue(UIA_ValueValuePropertyId, &mut var).ok()?;

            return variant_to_string(&var);
        }

        if is_firefox {
            let cond = automation
                .CreatePropertyCondition(UIA_NamePropertyId, "Search or enter address".into())
                .ok()?;

            let urlbar = element.FindFirst(TreeScope_Subtree, &cond).ok()?;

            let mut var = VARIANT::default();
            urlbar.GetCurrentPropertyValue(UIA_ValueValuePropertyId, &mut var).ok()?;

            return variant_to_string(&var);
        }

        None
    }
}

fn main() {
    println!("Aktív ablak: {}", get_active_window_title());

    match get_browser_url() {
        Some(url) => println!("URL: {}", url),
        None => println!("Nem böngésző vagy nem sikerült URL-t olvasni."),
    }
}
