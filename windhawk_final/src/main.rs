#![windows_subsystem = "windows"]
use std::process::Command;
use std::os::windows::process::CommandExt;

fn main() {
    let _ = Command::new("G:\\1AZEDDIG\\Windhawk\\windhawk.exe")
        .arg("-tray-only")
        .creation_flags(0x08000000)
        .spawn();
}
