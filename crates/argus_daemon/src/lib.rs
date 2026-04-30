use std::process::Command;
use sysinfo::System;

/// Hunts down any process running on a specific port and kills it
pub fn assassinate_port(port: u16) {
    println!("--> [DAEMON] Searching for zombie processes on port {}...", port);
    let cmd = format!("lsof -ti :{} | xargs kill -9", port);
    let output = Command::new("sh").arg("-c").arg(&cmd).output().expect("Failed to execute");

    if output.status.success() {
        println!("--> [DAEMON] SUCCESS: Port {} has been forcibly cleared.", port);
    } else {
        println!("--> [DAEMON] STATUS: Port {} is already empty.", port);
    }
}

/// Reads the live RAM usage of the system
pub fn report_memory() {
    println!("--> [DAEMON] Scanning system telemetry...");
    let mut sys = System::new_all();
    sys.refresh_memory();
    
    // Convert bytes to Megabytes for readability
    let total = sys.total_memory() / 1_048_576; 
    let used = sys.used_memory() / 1_048_576;
    
    println!("--> [DAEMON] Memory Usage: {} MB / {} MB", used, total);
}

/// Uses macOS native commands to launch any installed application
pub fn launch_app(app_name: &str) {
    println!("--> [DAEMON] Booting {}...", app_name);
    
    // 'open -a <Name>' is the Mac terminal command to launch apps
    let status = Command::new("open")
        .arg("-a")
        .arg(app_name)
        .status();

    if status.is_err() {
        println!("--> [DAEMON] ERROR: Could not find application '{}'", app_name);
    }
}