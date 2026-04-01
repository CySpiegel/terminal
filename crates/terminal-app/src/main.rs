use clap::Parser;

#[derive(Parser)]
#[command(name = "terminal", version, about = "GPU-rendered terminal emulator with built-in GTD")]
struct Cli {
    /// Start with GTD overlay visible
    #[arg(long)]
    gtd: bool,

    /// Config file path override
    #[arg(long, short)]
    config: Option<String>,

    /// Command to run instead of default shell
    #[arg(short = 'e', long)]
    command: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("terminal=info".parse()?),
        )
        .init();

    tracing::info!(
        "terminal v{} starting",
        env!("CARGO_PKG_VERSION")
    );

    // TODO: Initialize winit event loop
    // TODO: Create wgpu instance and window
    // TODO: Spawn terminal surface (PTY + read/write/render threads)
    // TODO: If cli.gtd, show GTD overlay on start
    // TODO: Enter main event loop

    println!("terminal v{}", env!("CARGO_PKG_VERSION"));
    println!("GPU-rendered terminal emulator with built-in GTD");
    println!();
    println!("This is a scaffold — the full implementation is in progress.");
    println!("Architecture: Ghostty-style per-terminal threads + wgpu rendering");

    Ok(())
}
