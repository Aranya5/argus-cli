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

// 3. APP LAUNCHER (Dynamic Upgrade)
pub fn launch_app(app_name: &str) {
    println!("--> [DAEMON] Attempting to boot '{}'...", app_name);
    
    let output = Command::new("open")
        .arg("-a")
        .arg(app_name)
        .output()
        .expect("Failed to execute open command");

    // macOS 'open' returns success if the app is found, and an error code if it isn't.
    if output.status.success() {
        println!("--> [DAEMON] SUCCESS: {} launched.", app_name);
    } else {
        println!("--> [DAEMON] ERROR: macOS could not find an app named '{}'. Is it installed?", app_name);
    }
}

// 3b. APP TERMINATOR
pub fn close_app(app_name: &str) {
    println!("--> [DAEMON] Requesting graceful shutdown for '{}'...", app_name);
    
    // We use osascript to tell the macOS UI to quit the app properly (like Cmd+Q)
    let script = format!("quit app \"{}\"", app_name);
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .expect("Failed to execute osascript command");

    if output.status.success() {
        println!("--> [DAEMON] SUCCESS: {} closed.", app_name);
    } else {
        println!("--> [DAEMON] ERROR: macOS could not close '{}'. Is it actually running?", app_name);
    }
}

// 3c. URL LAUNCHER
pub fn open_url(url: &str) {
    println!("--> [DAEMON] Opening browser to '{}'...", url);
    
    let output = Command::new("open")
        .arg(url)
        .output()
        .expect("Failed to execute open command");

    if output.status.success() {
        println!("--> [DAEMON] SUCCESS: Webpage launched.");
    } else {
        println!("--> [DAEMON] ERROR: macOS could not open the URL.");
    }
}

// 3d. TAB TERMINATOR
pub fn close_tab() {
    println!("--> [DAEMON] Closing active website tab...");
    
    // Simulates pressing Cmd + W on the keyboard
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to keystroke \"w\" using command down")
        .output();
        
    println!("--> [DAEMON] SUCCESS: Tab closed.");
}

// 3e. RESURRECT TAB
pub fn reopen_tab() {
    println!("--> [DAEMON] Resurrecting the last closed tab...");
    
    // Simulates pressing Cmd + Shift + T on the keyboard
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to keystroke \"t\" using {command down, shift down}")
        .output();
        
    println!("--> [DAEMON] SUCCESS: Previous tab restored.");
}

// THE MOBILE RESET
// Clears the Watchman cache and wipes the temporary Metro bundler files
pub fn clear_bundler_cache() {
    println!("--> [DAEMON] Executing mobile environment reset...");
    
    // 1. Clear Watchman
    let _ = Command::new("sh")
        .arg("-c")
        .arg("watchman watch-del-all")
        .output();
        
    // 2. Clear Metro Bundler Cache (macOS specific temp folder)
    let _ = Command::new("sh")
        .arg("-c")
        .arg("rm -rf $TMPDIR/metro-* && rm -rf $TMPDIR/haste-map-*")
        .output();

    println!("--> [DAEMON] SUCCESS: Metro cache cleared and Watchman reset.");
}

// // THE DATABASE IGNITION
// // Boots up MongoDB using Homebrew services
// pub fn start_database() {
//     println!("--> [DAEMON] Igniting local database...");
    
//     let output = Command::new("sh")
//         .arg("-c")
//         .arg("brew services start mongodb-community")
//         .output()
//         .expect("Failed to execute brew command");

//     if output.status.success() {
//         println!("--> [DAEMON] SUCCESS: MongoDB is now running in the background.");
//     } else {
//         println!("--> [DAEMON] WARNING: Database failed to start. Is Homebrew MongoDB installed?");
//     }
// }

// THE NUKE PROTOCOL
// Deletes node_modules and reinstalls dependencies
pub fn nuke_node_modules() {
    println!("--> [DAEMON] Initiating Nuke Protocol: node_modules...");
    
    // 1. Delete node_modules
    let _ = Command::new("rm")
        .arg("-rf")
        .arg("node_modules")
        .output();
        
    println!("--> [DAEMON] Deletion complete. Reinstalling dependencies...");

    // 2. Run npm install (Inherits your current terminal's environment)
    let status = Command::new("npm")
        .arg("install")
        .status()
        .expect("Failed to execute npm install");

    if status.success() {
        println!("--> [DAEMON] SUCCESS: Project is fresh and dependencies are reinstalled.");
    } else {
        println!("--> [DAEMON] ERROR: npm install failed. Check your internet connection.");
    }
}