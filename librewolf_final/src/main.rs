#![windows_subsystem = "windows"]

use std::process::Command;
use std::os::windows::process::CommandExt;
use std::env;

fn main() {
    // Argumentumok (ráhúzott fájlok) begyűjtése
    let args: Vec<String> = env::args().skip(1).collect();

    // A fix útvonal
    let mut cmd = Command::new("g:\\Mozilla\\Librewolf\\LibreWolf-Portable.exe");
    
    // Ha van ráhúzott fájl, átadjuk
    if !args.is_empty() {
        cmd.args(&args);
    }

    // Láthatatlan indítás
    let _ = cmd.creation_flags(0x08000000).spawn();
}
