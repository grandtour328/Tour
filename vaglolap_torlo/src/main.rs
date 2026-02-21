#![windows_subsystem = "windows"]

use windows_sys::Win32::System::DataExchange::{OpenClipboard, EmptyClipboard, CloseClipboard};

fn main() {
    unsafe {
        // Megpróbáljuk megnyitni a vágólapot (0 = az aktuális folyamathoz rendelve)
        if OpenClipboard(0) != 0 {
            // Kiürítjük a tartalmát
            EmptyClipboard();
            // Lezárjuk, hogy más programok is hozzáférjenek
            CloseClipboard();
        }
    }
}