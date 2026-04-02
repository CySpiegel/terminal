use clap::Parser;

#[derive(Parser)]
#[command(name = "terminal", version, about = "GPU-rendered terminal emulator with built-in GTD and AI assistant")]
struct Cli {
    /// Start with GTD overlay visible
    #[arg(long)]
    gtd: bool,

    /// Start with AI assistant active
    #[arg(long)]
    ai: bool,

    /// Config file path override
    #[arg(long, short)]
    config: Option<String>,

    /// Command to run instead of default shell
    #[arg(short = 'e', long)]
    command: Option<String>,

    /// Override AI model (e.g., "gpt-4o", "llama3.1:70b")
    #[arg(long)]
    model: Option<String>,

    /// Override AI API base URL (e.g., "http://localhost:11434/v1")
    #[arg(long)]
    api_url: Option<String>,
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
    // TODO: If cli.ai, activate AI assistant pane
    // TODO: Enter main event loop

    println!("terminal v{}", env!("CARGO_PKG_VERSION"));
    println!("GPU-rendered terminal emulator with built-in GTD and AI assistant");
    println!();
    println!("Architecture:");
    println!("  - Ghostty-style per-terminal threads (read/write/render)");
    println!("  - wgpu GPU rendering (Vulkan/Metal/DX12)");
    println!("  - Built-in GTD task management (SQLite, vi-keybindings)");
    println!("  - AI coding assistant (any OpenAI-compatible endpoint)");
    println!();

    if let Some(ref url) = cli.api_url {
        println!("  AI endpoint: {}", url);
    } else {
        println!("  AI endpoint: http://localhost:11434/v1 (default, Ollama)");
    }
    if let Some(ref model) = cli.model {
        println!("  AI model:    {}", model);
    }

    println!();
    println!("Configure in ~/.config/terminal/config.toml:");
    println!();
    println!("  [ai]");
    println!("  base_url = \"http://localhost:4000\"   # LiteLLM, Ollama, vLLM, etc.");
    println!("  model = \"llama3.1:70b\"");
    println!("  api_key = \"sk-...\"                   # optional for local models");

    Ok(())
}
