use anyhow::{Context, Result};
use clap::Parser;
use core::convert::Infallible;
use std::process::Stdio;
use tokio::{io::AsyncWriteExt as _, process::Command, signal::unix::SignalKind};
use tracing::{info, warn};

#[derive(Parser, Debug)]
struct Args {
    // currently required because we don't do much else
    // in the future we can have more options
    #[arg(long, required = true)]
    on_term_write_line: Vec<String>,

    #[arg(required = true)]
    command: String,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!(
        "Starting wrapper for command: {} with args: {:?}",
        args.command, args.args
    );

    let mut child = Command::new(&args.command)
        .args(&args.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn child process")?;

    let mut stdin = child.stdin.take().context("Failed to get child stdin")?;
    let handle = tokio::spawn(async move {
        let mut signals = tokio::signal::unix::signal(SignalKind::terminate())
            .expect("Failed to set up signal handler");

        while let Some(()) = signals.recv().await {
            info!(
                "Received SIGTERM, executing commands: {:?}",
                args.on_term_write_line
            );

            for cmd in &args.on_term_write_line {
                stdin.write_all(cmd.as_bytes()).await?;
                stdin.write_all(b"\n").await?;
            }

            stdin.flush().await?;
        }

        Err::<Infallible, _>(anyhow::anyhow!("Signal handler task ended unexpectedly"))
    });

    tokio::select! {
        res = child.wait() => {
            let status = res.context("Failed to wait for child process")?;
            info!("Child process exited with status: {}", status);
        }
        Err(err) = handle => {
            warn!("Signal handler task failed: {:?}", err);
        }
    };

    Ok(())
}
