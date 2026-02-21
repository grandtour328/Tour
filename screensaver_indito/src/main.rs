#![windows_subsystem = "windows"]

use std::process::Command;
use std::os::windows::process::CommandExt;

fn main() {
    // Az /s kapcsoló elindítja a kímélőt (nélküle csak a beállítások nyílnának meg)
    let _ = Command::new("c:\\Windows\\System32\\ssText3d.scr")
        .arg("/s")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn();
}
