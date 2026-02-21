#![windows_subsystem = "windows"]

use std::process::Command;
use std::os::windows::process::CommandExt;
use std::env;

fn main() {
    // Begyűjtjük a Windows-tól kapott fájlútvonalat (ha van)
    let args: Vec<String> = env::args().skip(1).collect();

    let mut cmd = Command::new("notepad.exe");
    
    // Ha húztál rá fájlt, továbbadjuk a Notepadnek
    if !args.is_empty() {
        cmd.args(&args);
    }

    // Indítás láthatatlanul
    let _ = cmd.creation_flags(0x08000000).spawn();
}
