use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
fn test_basic_execution() {
    // Test that tsw can execute a basic command
    let output = Command::new("cargo")
        .args(&["run", "--", "echo", "test"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_signal_handling() {
    // Build the project first
    let build = Command::new("cargo")
        .args(&["build"])
        .output()
        .expect("Failed to build");
    assert!(build.status.success());

    // Start tsw with mock server
    let child = Command::new("./target/debug/tsw")
        .args(&[
            "--on-int-write=save-all",
            "--on-int-write=quit",
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

    // Send SIGINT, then wait for the process to exit
    signal::kill(pid, Signal::SIGINT).expect("Failed to send SIGINT");

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

#[test]
fn test_sigterm_handling() {
    // Build the project first
    let build = Command::new("cargo")
        .args(&["build"])
        .output()
        .expect("Failed to build");
    assert!(build.status.success());

    // Start tsw with mock server
    let child = Command::new("./target/debug/tsw")
        .args(&[
            "--on-int-write=save-all",
            "--on-int-write=quit",
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

    // Send SIGTERM (systemd default), then wait for the process to exit
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
        "Expected 'Received: save-all' in stdout (SIGTERM should work like SIGINT)"
    );
    assert!(
        stdout.contains("Received: quit"),
        "Expected 'Received: quit' in stdout (SIGTERM should work like SIGINT)"
    );
}
