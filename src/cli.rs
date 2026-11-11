//! CLI module - extracted from main.rs for backward compatibility
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::fs;
use dirs;

#[derive(Parser)]
#[command(name = "reset-launcher")]
#[command(about = "Offline launcher for Reset (Minecraft Rust Edition)", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build the Reset client in release mode
    Build {
        /// Path to the Reset repository
        #[arg(long, default_value = "../reset-client")]
        reset_path: PathBuf,
    },
    /// Prepare game resources from a Minecraft server.jar (requires JDK 25)
    Prepare {
        /// Path to the Reset repository
        #[arg(long, default_value = "../reset-client")]
        reset_path: PathBuf,
        /// Path to the Minecraft server.jar file
        #[arg(long)]
        server_jar: PathBuf,
        /// Also extract sounds
        #[arg(long)]
        sounds: bool,
    },
    /// Launch the Reset client in offline mode
    Launch {
        /// Path to the Reset repository
        #[arg(long, default_value = "../reset-client")]
        reset_path: PathBuf,
        /// Player name (default: generated offline name)
        #[arg(long)]
        name: Option<String>,
        /// Game directory (saves, options.txt, etc.)
        #[arg(long)]
        game_dir: Option<PathBuf>,
        /// Resource directory (prepared by reset-prepare)
        #[arg(long)]
        resources: Option<PathBuf>,
        /// Render distance (2-128)
        #[arg(long, default_value = "12")]
        render_distance: i32,
        /// Window width
        #[arg(long, default_value = "1280")]
        width: u32,
        /// Window height
        #[arg(long, default_value = "720")]
        height: u32,
        /// Fullscreen mode
        #[arg(long)]
        fullscreen: bool,
        /// World name to load/create
        #[arg(long)]
        world: Option<String>,
        /// World seed
        #[arg(long)]
        seed: Option<i64>,
        /// Creative mode
        #[arg(long)]
        creative: bool,
        /// Join a server at startup (offline mode)
        #[arg(long)]
        server: Option<String>,
        /// Run headless (no window)
        #[arg(long)]
        headless: bool,
        /// Custom arguments passed directly to reset-client
        #[arg(last = true)]
        extra_args: Vec<String>,
    },
    /// Check if the client and resources are ready
    Check {
        /// Path to the Reset repository
        #[arg(long, default_value = "../reset-client")]
        reset_path: PathBuf,
    },
}

#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub name: Option<String>,
    pub game_dir: Option<PathBuf>,
    pub resources: Option<PathBuf>,
    pub render_distance: i32,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub world: Option<String>,
    pub seed: Option<i64>,
    pub creative: bool,
    pub server: Option<String>,
    pub headless: bool,
    pub extra_args: Vec<String>,
}

/// Run the CLI application
pub async fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { reset_path } => build_client(&reset_path).await,
        Commands::Prepare { reset_path, server_jar, sounds } => prepare_resources(&reset_path, &server_jar, sounds).await,
        Commands::Launch { reset_path, name, game_dir, resources, render_distance, width, height, fullscreen, world, seed, creative, server, headless, extra_args } => {
            launch_client(&reset_path, LaunchOptions {
                name,
                game_dir,
                resources,
                render_distance,
                width,
                height,
                fullscreen,
                world,
                seed,
                creative,
                server,
                headless,
                extra_args,
            }).await
        }
        Commands::Check { reset_path } => check_ready(&reset_path).await,
    }
}

async fn build_client(reset_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Building Reset client in release mode...");
    println!("Repository path: {}", reset_path.display());

    let client_path = reset_path.join("crates").join("reset-client");
    if !client_path.exists() {
        return Err(format!("reset-client not found at {}", client_path.display()).into());
    }

    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "reset-client"])
        .current_dir(reset_path)
        .status()?;

    if !status.success() {
        return Err("Build failed".into());
    }

    println!("Build successful!");
    println!("Binary location: {}", reset_path.join("target").join("release").join("reset-client").display());
    Ok(())
}

async fn prepare_resources(reset_path: &Path, server_jar: &Path, sounds: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Preparing Reset resources...");
    println!("Repository path: {}", reset_path.display());
    println!("Server JAR: {}", server_jar.display());

    if !server_jar.exists() {
        return Err(format!("server.jar not found at {}", server_jar.display()).into());
    }

    let prepare_path = reset_path.join("crates").join("reset-prepare");
    if !prepare_path.exists() {
        return Err(format!("reset-prepare not found at {}", prepare_path.display()).into());
    }

    let mut args = vec!["run", "--release", "-p", "reset-prepare", "--", "--prepare", "--server-jar"];
    args.push(server_jar.to_str().unwrap());

    if sounds {
        args.push("--complete");
    }

    let status = Command::new("cargo")
        .args(&args)
        .current_dir(reset_path)
        .status()?;

    if !status.success() {
        return Err("Resource preparation failed".into());
    }

    println!("Resource preparation successful!");
    let resources_dir = reset_path.join(".local").join("reset-client").join("26.3");
    println!("Resources location: {}", resources_dir.display());
    Ok(())
}

async fn launch_client(reset_path: &Path, opts: LaunchOptions) -> Result<(), Box<dyn std::error::Error>> {
    println!("Launching Reset client...");

    // Find the binary
    let binary = reset_path.join("target").join("release").join("reset-client");
    #[cfg(windows)]
    let binary = binary.with_extension("exe");

    if !binary.exists() {
        return Err(format!(
            "Client binary not found at {}. Run 'reset-launcher build' first.",
            binary.display()
        ).into());
    }

    // Determine resources directory
    let resources = opts.resources.unwrap_or_else(|| {
        reset_path.join(".local").join("reset-client").join("26.3")
    });

    if !resources.join("reset-resources.json").exists() {
        eprintln!("WARNING: Resources not found at {}", resources.display());
        eprintln!("Run 'reset-launcher prepare --server-jar <path>' first.");
        eprintln!("You need a Minecraft server.jar and JDK 25 to prepare resources.");
    }

    // Determine game directory
    let game_dir = opts.game_dir.unwrap_or_else(|| {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("reset-offline")
    });

    fs::create_dir_all(&game_dir)?;

    // Build arguments
    let mut args = vec![
        "--resources".to_string(), resources.to_string_lossy().to_string(),
        "--render-distance".to_string(), opts.render_distance.to_string(),
        "--width".to_string(), opts.width.to_string(),
        "--height".to_string(), opts.height.to_string(),
        "--game-dir".to_string(), game_dir.to_string_lossy().to_string(),
    ];

    if opts.fullscreen {
        args.push("--fullscreen".to_string());
    }

    if opts.headless {
        args.push("--headless".to_string());
    }

    if let Some(name) = opts.name {
        args.push("--name".to_string());
        args.push(name);
    }

    if let Some(world) = opts.world {
        args.push("--world".to_string());
        args.push(world);
    }

    if let Some(seed) = opts.seed {
        args.push("--seed".to_string());
        args.push(seed.to_string());
    }

    if opts.creative {
        args.push("--creative".to_string());
    }

    if let Some(server) = opts.server {
        args.push("--server".to_string());
        args.push(server);
    }

    // Add extra arguments
    args.extend(opts.extra_args);

    println!("Launching: {} {}", binary.display(), args.join(" "));

    let mut cmd = Command::new(&binary);
    cmd.args(&args)
        .current_dir(&game_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status()?;

    if !status.success() {
        eprintln!("Client exited with status: {:?}", status.code());
    }

    Ok(())
}

async fn check_ready(reset_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking Reset offline launcher readiness...\n");

    // Check binary
    let binary = reset_path.join("target").join("release").join("reset-client");
    #[cfg(windows)]
    let binary = binary.with_extension("exe");

    if binary.exists() {
        println!("✓ Client binary: {}", binary.display());
    } else {
        println!("✗ Client binary: NOT FOUND (run 'build')");
    }

    // Check resources
    let resources = reset_path.join(".local").join("reset-client").join("26.3");
    if resources.join("reset-resources.json").exists() {
        println!("✓ Resources: {}", resources.display());
    } else {
        println!("✗ Resources: NOT FOUND (run 'prepare' with a server.jar)");
        println!("  Required: Minecraft server.jar + JDK 25");
    }

    // Check game directory
    let game_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("reset-offline");
    if game_dir.exists() {
        println!("✓ Game directory: {}", game_dir.display());
    } else {
        println!("○ Game directory: Will be created at {}", game_dir.display());
    }

    println!("\nTo get started:");
    println!("  1. Run 'reset-launcher build' to build the client");
    println!("  2. Run 'reset-launcher prepare --server-jar <path>' to prepare resources");
    println!("  3. Run 'reset-launcher launch' to play offline");

    Ok(())
}