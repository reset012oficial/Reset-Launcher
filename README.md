# Horizon Offline Launcher

A simple offline launcher for **Horizon** (Minecraft Rust Edition) - a clean-room rewrite of Minecraft Java Edition 26.3 in Rust.

## Prerequisites

Before using this launcher, you need to install:

1. **Rust 1.90+** - [Install from rustup.rs](https://rustup.rs/)
2. **JDK 25** - Required for resource preparation
3. **Minecraft: Java Edition** - You need a legitimate copy (server.jar)

## Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# On Windows, download and run rustup-init.exe from https://rustup.rs/

# Restart your terminal after installation
rustc --version  # Should show 1.90+
```

## Quick Start

### 1. Clone the Horizon repository
```bash
git clone https://github.com/horizon-rust/horizon.git
cd horizon
```

### 2. Build the offline launcher
```bash
cd ../horizon-offline-launcher
cargo build --release
```

### 3. Build the Horizon client
```bash
./target/release/horizon-offline-launcher build --horizon-path ../horizon
```

### 4. Prepare game resources (requires server.jar and JDK 25)
```bash
# Get server.jar from your Minecraft launcher (Installations > Java Edition > Version > Server)
# Or download from https://www.minecraft.net/en-us/download/server

./target/release/horizon-offline-launcher prepare \
  --horizon-path ../horizon \
  --server-jar /path/to/server.jar \
  --sounds  # Optional: also extract sound files
```

### 5. Launch offline!
```bash
# Single player
./target/release/horizon-offline-launcher launch --horizon-path ../horizon

# With custom settings
./target/release/horizon-offline-launcher launch \
  --horizon-path ../horizon \
  --name "Steve" \
  --world "MyWorld" \
  --render-distance 16 \
  --width 1920 --height 1080 \
  --fullscreen

# Join a LAN/offline server
./target/release/horizon-offline-launcher launch \
  --horizon-path ../horizon \
  --server localhost:25565 \
  --name "Player1"
```

### 6. Check status
```bash
./target/release/horizon-offline-launcher check --horizon-path ../horizon
```

## Commands

| Command | Description |
|---------|-------------|
| `build` | Build the Horizon client in release mode |
| `prepare` | Prepare game resources from server.jar (requires JDK 25) |
| `launch` | Launch the Horizon client in offline mode |
| `check` | Check if client and resources are ready |

## Launch Options

```
USAGE:
    horizon-offline-launcher launch [OPTIONS] [EXTRA_ARGS]...

OPTIONS:
    --horizon-path <PATH>       Path to Horizon repository [default: ../horizon/horizon-main]
    --name <NAME>               Player name (default: generated offline name)
    --game-dir <DIR>            Game directory (saves, options.txt, etc.)
    --resources <DIR>           Resource directory (prepared by horizon-prepare)
    --render-distance <N>       Chunks, 2..128 [default: 12]
    --width <N>                 Window width [default: 1280]
    --height <N>                Window height [default: 720]
    --fullscreen                Open window fullscreen
    --world <NAME>              World folder under saves/
    --seed <N>                  World seed for new worlds
    --creative                  Creative mode (default survival)
    --server <HOST[:PORT]>      Join multiplayer server (offline mode)
    --headless                  Run headless (no window)
    --help                      Print help
    -h                          Print help (short)
```

## How It Works

1. **Build**: Compiles `horizon-client` from the Horizon repository using Cargo
2. **Prepare**: Runs `horizon-prepare` which:
   - Imports and checks the pinned Minecraft jars
   - Runs vanilla exporters to extract assets
   - Writes the resource folder (textures, models, sounds, etc.)
3. **Launch**: Runs the compiled `horizon-client` binary with offline-friendly arguments

## Requirements for Resource Preparation

The `prepare` command requires:
- **JDK 25** (Java Development Kit 25) - not just JRE
- **Minecraft server.jar** from your legitimate Java Edition copy
- The server.jar must match the version Horizon targets (currently 26.3)

## Game Directory

By default, the launcher uses:
- Windows: `%APPDATA%\horizon-offline\`
- Linux: `~/.local/share/horizon-offline/`
- macOS: `~/Library/Application Support/horizon-offline/`

This directory contains:
- `saves/` - Your single-player worlds
- `options.txt` - Game settings
- `resourcepacks/` - Resource packs
- `mods/` - Native Rust mods (.hmod files)

## Troubleshooting

### "Resources not found"
Run the `prepare` command with a valid server.jar and JDK 25.

### "Client binary not found"
Run the `build` command first.

### "JDK 25 not found"
Make sure JDK 25 is installed and `java` is in your PATH. The `horizon-prepare` crate uses Java to run the vanilla asset exporters.

### Build fails
- Ensure you have Rust 1.90+
- Try `cargo clean` in the Horizon repository and rebuild
- Check that all dependencies are available (especially on Linux: `libasound2-dev libudev-dev`)

## Legal Notice

This launcher is for **Horizon**, a clean-room rewrite of Minecraft Java Edition. You must own a legitimate copy of Minecraft: Java Edition to prepare the game resources. The launcher does not distribute any Minecraft assets.

## License

MIT License - See LICENSE file for details.