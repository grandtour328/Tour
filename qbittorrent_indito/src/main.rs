#![windows_subsystem = "windows"]

use std::process::Command;
use std::os::windows::process::CommandExt;
use std::{thread, time::Duration};

fn main() {
    // 120 másodperces várakozás
    thread::sleep(Duration::from_secs(120));

    // Indítás láthatatlanul (maga a qBittorrent persze meg fog jelenni)
    let _ = Command::new("g:\\AAAA\\TORRENT\\Qbittorrent\\qbittorrent.exe")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn();
}
