use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(about = "Terraria Server Wrapper - A simple wrapper for interactive servers")]
struct Args {
    /// Commands to write to child process on SIGINT/SIGTERM (can be specified multiple times)
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

    // Set up signal handling with channel for immediate notification
    let (signal_tx, signal_rx) = mpsc::channel();
    let on_int_write = args.on_int_write.clone();

    // Register signal handlers for both SIGINT and SIGTERM
    // Use signal_hook's iterator API for better signaling
    let mut signals = signal_hook::iterator::Signals::new([
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
    ])?;

    let signal_tx_clone = signal_tx.clone();
    std::thread::spawn(move || {
        if let Some(sig) = signals.forever().next() {
            info!("Received signal: {}", sig);
            let _ = signal_tx_clone.send(());
        }
    });

    // Monitor for shutdown signal - only spawn if there are commands to send
    let stdin_handle = if !on_int_write.is_empty() {
        Some(std::thread::spawn(move || {
            let mut stdin = child_stdin;

            // Wait for shutdown signal
            if signal_rx.recv().is_ok() {
                // When signal is received, send the on-int-write commands
                info!("Shutdown signal received, sending commands to child process");
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
            }
        }))
    } else {
        None
    };

    // Wait for the child process to exit
    let status = child.wait().context("Failed to wait for child process")?;

    // If monitoring thread exists, ensure it completes
    if let Some(handle) = stdin_handle {
        // Signal shutdown in case child exited before signal was received
        let _ = signal_tx.send(());

        // Wait for the monitoring thread to finish
        if handle.join().is_err() {
            warn!("Monitoring thread panicked while shutting down");
        }
    }

    info!("Child process exited with status: {}", status);

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Child process exited with non-zero status: {}", status)
    }
}
