// crates/argus_voice/src/router.rs

use crate::mappers;
use std::fs::OpenOptions;
use std::io::Write;

// THE CROSS-TERMINAL LOGGER
pub fn write_log(message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/argus.log") 
    {
        let log_entry = format!("{}\n", message);
        let _ = file.write_all(log_entry.as_bytes());
    }
}

pub fn execute(command: &str) {
    println!("--> Argus Executing: '{}'", command);

    // 1. SANITIZE INPUT
    let clean_cmd = command
        .replace("argus", "")
        .replace("august", "")
        .trim()
        .to_string();

    // Log the sanitized command to the TUI Dashboard immediately
    write_log(&format!("[VOICE] Heard: '{}'", clean_cmd));

    let is_port_hit = clean_cmd.contains("kill port")
        || clean_cmd.contains("clear port")
        || clean_cmd.contains("close port")
        || clean_cmd.contains("terminate port");

    // 2. ROUTE THE COMMAND

    // PORT KILLER
    if is_port_hit {
        if let Some(port) = mappers::extract_dynamic_port(&clean_cmd) {
            println!("--> ACTION: Initiating termination protocol for port {}...", port);
            write_log(&format!("[ACTION] Terminating port {}...", port));
            argus_daemon::assassinate_port(port);
        } else {
            println!("--> [DAEMON] ERROR: Couldn't understand the port number.");
            write_log("[ERROR] Could not parse port number.");
        }
    }
    // TELEMETRY
    else if clean_cmd.contains("system memory") {
        println!("--> ACTION: Reading telemetry...");
        write_log("[ACTION] Scanning system telemetry...");
        argus_daemon::report_memory();
    }
    // TAB / SITE RESURRECTOR 
    else if clean_cmd.contains("last") || clean_cmd.contains("previous") || clean_cmd.contains("just closed") || clean_cmd.contains("reopen site") || clean_cmd.contains("reopen tab"){
        write_log("[ACTION] Resurrecting previous tab...");
        argus_daemon::reopen_tab();
    }
    // URL LAUNCHER
    else if clean_cmd.contains("open site ") || clean_cmd.contains("open tab ") {
        let target = clean_cmd.replace("open site", "").replace("open tab", "").trim().to_string();

        if let Some(actual_url) = mappers::map_url(&target) {
            write_log(&format!("[ACTION] Opening URL: {}", target));
            argus_daemon::open_url(actual_url);
        } else {
            println!("--> [DAEMON] ERROR: I don't have a URL mapped for '{}'.", target);
            write_log(&format!("[ERROR] Unmapped URL target: '{}'", target));
        }
    }
    // TAB / SITE TERMINATOR
    else if clean_cmd.contains("close site") || clean_cmd.contains("close tab") {
        write_log("[ACTION] Closing current browser tab...");
        argus_daemon::close_tab();
    }
    // APP LAUNCHER
    else if clean_cmd.contains("open ") {
        let target = clean_cmd.replace("open", "").trim().to_string();

        if !target.is_empty() {
            write_log(&format!("[ACTION] Launching App: {}", target));
            argus_daemon::launch_app(&target);
        }
    }
    // APP TERMINATOR
    else if clean_cmd.contains("close ") && !clean_cmd.contains("port") {
        let target = clean_cmd.replace("close", "").trim().to_string();

        if !target.is_empty() {
            write_log(&format!("[ACTION] Closing App: {}", target));
            argus_daemon::close_app(&target);
        }
    }
    // DEV TOOLS: METRO RESET
    else if clean_cmd.contains("clear") && clean_cmd.contains("cache") {
        println!("--> ACTION: Nuke protocol authorized. Clearing bundler cache...");
        write_log("[ACTION] Clearing Metro/Watchman cache...");
        argus_daemon::clear_bundler_cache();
    }
    // DEV TOOLS: NODE MODULES
    else if clean_cmd.contains("nuke") && clean_cmd.contains("node") {
        println!("--> ACTION: Nuke protocol authorized. Rebuilding project...");
        write_log("[ACTION] Nuking node_modules & reinstalling...");
        argus_daemon::nuke_node_modules();
    }
    // SYSTEM: SLEEP
    else if clean_cmd.contains("sleep") && !clean_cmd.contains("port") {
        println!("--> ACTION: Going dormant...");
        write_log("[SYSTEM] Going dormant...");
    }
    // FALLBACK CATCH
    else {
        println!("--> [DAEMON] WARNING: Command parsed, but no routing logic found for '{}'", clean_cmd);
        write_log("[WARNING] Command not recognized by router.");
    }
}