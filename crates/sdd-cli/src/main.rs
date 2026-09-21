use clap::Parser;

#[derive(Parser)]
#[command(name = "sdd")]
#[command(about = "SDD AI Development Harness")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Init,
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            tracing::info!("Initializing SDD project...");
        }
        Some(Commands::Status) => {
            tracing::info!("Project status...");
        }
        None => {
            tracing::info!("SDD AI Development Harness");
        }
    }

    Ok(())
}
