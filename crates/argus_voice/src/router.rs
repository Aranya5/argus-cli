// crates/argus_voice/src/router.rs

use crate::mappers;

pub fn execute(command: &str) {
    println!("--> Argus Executing: '{}'", command);

    // 1. SANITIZE INPUT
    // Strip the wake words immediately so we don't have to repeat this in every block
    let clean_cmd = command
        .replace("argus", "")
        .replace("august", "")
        .trim()
        .to_string();

    let is_port_hit = clean_cmd.contains("kill port")
        || clean_cmd.contains("clear port")
        || clean_cmd.contains("close port")
        || clean_cmd.contains("terminate port");

    // 2. ROUTE THE COMMAND

    // PORT KILLER
    if is_port_hit {
        if let Some(port) = mappers::extract_dynamic_port(&clean_cmd) {
            println!(
                "--> ACTION: Initiating termination protocol for port {}...",
                port
            );
            argus_daemon::assassinate_port(port);
        } else {
            println!("--> [DAEMON] ERROR: Couldn't understand the port number.");
        }
    }
    // TELEMETRY
    else if clean_cmd.contains("system memory") {
        println!("--> ACTION: Reading telemetry...");
        argus_daemon::report_memory();
    }
    // TAB / SITE RESURRECTOR (Must come before Open/Close App)
    else if clean_cmd.contains("last") || clean_cmd.contains("previous") || clean_cmd.contains("just closed") || clean_cmd.contains("reopen site") || clean_cmd.contains("reopen tab"){
        argus_daemon::reopen_tab();
    }
    // URL LAUNCHER (Must come before App Launcher)
    else if clean_cmd.contains("open site ") || clean_cmd.contains("open tab ") {
        let target = clean_cmd.replace("open site", "").replace("open tab", "").trim().to_string();

        if let Some(actual_url) = mappers::map_url(&target) {
            argus_daemon::open_url(actual_url);
        } else {
            println!(
                "--> [DAEMON] ERROR: I don't have a URL mapped for '{}'.",
                target
            );
        }
    }
    // TAB / SITE TERMINATOR (Must come before App Terminator)
    else if clean_cmd.contains("close site") || clean_cmd.contains("close tab") {
        argus_daemon::close_tab();
    }
    // APP LAUNCHER
    else if clean_cmd.contains("open ") {
        let target = clean_cmd.replace("open", "").trim().to_string();

        if !target.is_empty() {
            argus_daemon::launch_app(&target);
        }
    }
    // APP TERMINATOR
    else if clean_cmd.contains("close ") && !clean_cmd.contains("port") {
        let target = clean_cmd.replace("close", "").trim().to_string();

        if !target.is_empty() {
            argus_daemon::close_app(&target);
        }
    }
    // DEV TOOLS: METRO RESET
    else if clean_cmd.contains("clear") && clean_cmd.contains("cache") {
        println!("--> ACTION: Nuke protocol authorized. Clearing bundler cache...");
        argus_daemon::clear_bundler_cache();
    }
    // DEV TOOLS: NODE MODULES
    else if clean_cmd.contains("nuke") && clean_cmd.contains("node") {
        println!("--> ACTION: Nuke protocol authorized. Rebuilding project...");
        argus_daemon::nuke_node_modules();
    }
    // SYSTEM: SLEEP
    else if clean_cmd.contains("sleep") && !clean_cmd.contains("port") {
        println!("--> ACTION: Going dormant...");
    }
    // FALLBACK CATCH
    else {
        println!(
            "--> [DAEMON] WARNING: Command parsed, but no routing logic found for '{}'",
            clean_cmd
        );
    }
}
