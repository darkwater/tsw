use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
fn test_signal_handling() {
    // Build the project first
    let build = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("Failed to build");
    assert!(build.status.success());

    // Start tsw with mock server
    let child = Command::new("./target/debug/tsw")
        .args([
            "--on-term-write-line=save-all",
            "--on-term-write-line=quit",
            "bash",
            "tests/mock_server.sh",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn tsw");

    // Capture PID before moving `child` into `wait_with_output`
    let pid = Pid::from_raw(child.id() as i32);

    // Give it time to start
    thread::sleep(Duration::from_secs(1));

    // Send SIGTERM, then wait for the process to exit
    signal::kill(pid, Signal::SIGTERM).expect("Failed to send SIGTERM");

    // Wait for process to exit
    let output = child.wait_with_output().expect("Failed to wait for child");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    // Verify that the commands were sent to stdout
    assert!(
        stdout.contains("Received: save-all"),
        "Expected 'Received: save-all' in stdout"
    );
    assert!(
        stdout.contains("Received: quit"),
        "Expected 'Received: quit' in stdout"
    );
}
