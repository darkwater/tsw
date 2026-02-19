use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(about = "Terraria Server Wrapper - A simple wrapper for interactive servers")]
struct Args {
    /// Commands to write to child process on SIGINT (can be specified multiple times)
    #[arg(long = "on-int-write")]
    on_int_write: Vec<String>,

    /// The command to execute
    #[arg(required = true)]
    command: String,

    /// Arguments to pass to the command
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!(
        "Starting wrapper for command: {} with args: {:?}",
        args.command, args.args
    );

    // Spawn the child process
    let mut child = Command::new(&args.command)
        .args(&args.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn child process")?;

    let child_stdin = child.stdin.take().context("Failed to get child stdin")?;

    // Set up signal handling for SIGINT
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();
    let on_int_write = args.on_int_write.clone();

    // Register signal handlers for both SIGINT and SIGTERM
    signal_hook::flag::register(signal_hook::consts::SIGINT, shutdown_clone.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, shutdown_clone.clone())?;

    // Monitor for shutdown signal
    let stdin_handle = std::thread::spawn(move || {
        let mut stdin = child_stdin;
        while !shutdown_clone.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // When SIGINT is received, send the on-int-write commands
        info!("SIGINT received, sending commands to child process");
        for cmd in &on_int_write {
            info!("Writing command: {}", cmd);
            if let Err(e) = writeln!(stdin, "{}", cmd) {
                warn!("Failed to write command '{}': {}", cmd, e);
            }
            if let Err(e) = stdin.flush() {
                warn!("Failed to flush stdin: {}", e);
            }
            // Give the child process time to process each command
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    // Wait for the child process to exit
    let status = child.wait().context("Failed to wait for child process")?;

    // Signal the monitoring thread to exit if it hasn't already
    shutdown.store(true, Ordering::Relaxed);
    // Wait for the monitoring thread to finish
    let _ = stdin_handle.join();

    info!("Child process exited with status: {}", status);

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Child process exited with non-zero status: {}", status)
    }
}
