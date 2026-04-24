use std::process::Command;

/// Hunts down any process running on a specific port and kills it
pub fn assassinate_port(port: u16) {
    println!("--> [DAEMON] Searching for zombie processes on port {}...", port);

    // On Mac/Linux, we use 'lsof' to find the Process ID (PID), then pipe it to 'kill -9'
    let cmd = format!("lsof -ti :{} | xargs kill -9", port);

    let output = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .expect("Failed to execute terminal command");

    if output.status.success() {
        println!("--> [DAEMON] SUCCESS: Port {} has been forcibly cleared.", port);
    } else {
        println!("--> [DAEMON] STATUS: Port {} is already empty. No targets found.", port);
    }
}
