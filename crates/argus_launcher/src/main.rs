use std::process::{Command, Stdio};

fn main() {
    println!("Booting Argus Core System...");

    // 1. Start the Voice Engine as a hidden background child process
    // We pipe stdout to null so its println! statements don't corrupt the TUI graphics
    let mut voice_process = Command::new("cargo")
        .args(["run", "-p", "argus_voice"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start Argus Voice Engine");

    // 2. Start the TUI Dashboard in the foreground
    // This will hijack the terminal and run until the user presses 'q'
    let mut tui_process = Command::new("cargo")
        .args(["run", "-p", "argus_tui"])
        .spawn()
        .expect("Failed to start Argus TUI");

    // 3. Block this script and wait for the TUI to be closed by the user
    let _ = tui_process.wait();

    // 4. Teardown: When the TUI closes, assassinate the background Voice Engine
    println!("TUI closed. Shutting down background voice daemon...");
    let _ = voice_process.kill();
    let _ = voice_process.wait(); // Ensure it is fully dead

    println!("Argus safely powered down.");
}