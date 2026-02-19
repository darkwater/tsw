# tsw

Terraria Server Wrapper - A simple wrapper for interactive servers

## Overview

TSW (Terraria Server Wrapper) is a utility that wraps interactive server processes and handles graceful shutdowns. It was designed for Terraria servers but can be used with any interactive server that requires commands to be sent before shutdown.

## Problem

Running a Terraria server typically requires using `tmux` or `screen` to access the console, primarily because you need to send commands like `save-all` before quitting. This makes it difficult to manage the server with modern service managers like systemd.

## Solution

TSW acts as a wrapper that:
1. Spawns your server process
2. Listens for SIGINT signals (Ctrl+C or systemd stop)
3. Sends configured commands to the server's stdin before shutdown
4. Waits for the server to exit gracefully

## Installation

```bash
cargo build --release
# Binary will be at target/release/tsw
```

## Usage

Basic usage:
```bash
tsw --on-int-write="save-all" --on-int-write="quit" TerrariaServer -config serverconfig.txt
```

With systemd:
```ini
[Unit]
Description=Terraria Server
After=network.target

[Service]
Type=simple
User=terraria
WorkingDirectory=/opt/terraria
ExecStart=/usr/local/bin/tsw --on-int-write="save-all" --on-int-write="quit" TerrariaServer -config serverconfig.txt
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

## Options

- `--on-int-write <COMMAND>`: Command to write to the child process's stdin when SIGINT is received. Can be specified multiple times, and commands will be sent in order.

## Dependencies

- clap - Command-line argument parsing
- anyhow - Error handling
- tracing - Logging
- signal-hook - Signal handling

## Testing

Run the test suite:
```bash
cargo test
```

## License

This project is open source.