#![windows_subsystem = "windows"]

use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

fn main() {
    unsafe {
        // CTRL lenyomása
        keybd_event(VK_CONTROL as u8, 0, 0, 0);
        // ESC lenyomása
        keybd_event(VK_ESCAPE as u8, 0, 0, 0);
        
        // ESC felengedése
        keybd_event(VK_ESCAPE as u8, 0, KEYEVENTF_KEYUP, 0);
        // CTRL felengedése
        keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
    }
}
