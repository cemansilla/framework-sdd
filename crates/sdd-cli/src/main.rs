mod commands;

use clap::Parser;
use std::path::PathBuf;

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
    /// Initialize a new SDD project
    Init {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Show project status
    Status {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Start a brainstorming session
    Brainstorm {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Session title
        #[arg(short, long)]
        title: Option<String>,
    },
    /// Manage project requirements
    Requirements {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// List all requirements
        #[arg(short, long)]
        list: bool,
        /// Add a new requirement
        #[arg(short, long)]
        add: Option<String>,
    },
    /// Manage project questions
    Questions {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// List all questions
        #[arg(short, long)]
        list: bool,
        /// Add a new question
        #[arg(short, long)]
        add: Option<String>,
    },
    /// Manage project architecture
    Architecture {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Show current architecture
        #[arg(short, long)]
        show: bool,
        /// Define architecture style
        #[arg(short, long)]
        style: Option<String>,
    },
    /// Manage technical design
    Design {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Show current technical design
        #[arg(short, long)]
        show: bool,
    },
    /// Manage project tasks
    Tasks {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// List all tasks
        #[arg(short, long)]
        list: bool,
        /// Task ID to show details
        task_id: Option<String>,
    },
    /// Show context for a task
    Context {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Task ID
        task_id: String,
    },
    /// Implement a task
    Implement {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Task ID
        task_id: String,
    },
    /// Test a task
    Test {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Task ID
        task_id: String,
    },
    /// Review a task
    Review {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Task ID
        task_id: String,
    },
    /// Show change details
    Change {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Change ID
        change_id: Option<String>,
    },
    /// Analyze impact of a change
    Impact {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Change ID
        change_id: String,
    },
    /// Validate the project
    Validate {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Show project changelog
    Changelog {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Manage project hooks
    Hooks {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// List installed hooks
        #[arg(short, long)]
        list: bool,
        /// Install project hooks
        #[arg(short, long)]
        install: bool,
    },
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
        Some(Commands::Init { path }) => {
            commands::init::execute(path).await?;
        }
        Some(Commands::Status { path }) => {
            commands::status::execute(path).await?;
        }
        Some(Commands::Brainstorm { path, title }) => {
            commands::brainstorm::execute(path, title).await?;
        }
        Some(Commands::Requirements { path, list, add }) => {
            commands::requirements::execute(path, list, add).await?;
        }
        Some(Commands::Questions { path, list, add }) => {
            commands::questions::execute(path, list, add).await?;
        }
        Some(Commands::Architecture { path, show, style }) => {
            commands::architecture::execute(path, show, style).await?;
        }
        Some(Commands::Design { path, show }) => {
            commands::design::execute(path, show).await?;
        }
        Some(Commands::Tasks {
            path,
            list,
            task_id,
        }) => {
            commands::tasks::execute(path, list, task_id).await?;
        }
        Some(Commands::Context { path, task_id }) => {
            commands::context::execute(path, task_id).await?;
        }
        Some(Commands::Implement { path, task_id }) => {
            commands::implement::execute(path, task_id).await?;
        }
        Some(Commands::Test { path, task_id }) => {
            commands::test::execute(path, task_id).await?;
        }
        Some(Commands::Review { path, task_id }) => {
            commands::review::execute(path, task_id).await?;
        }
        Some(Commands::Change { path, change_id }) => {
            commands::change::execute(path, change_id).await?;
        }
        Some(Commands::Impact { path, change_id }) => {
            commands::impact::execute(path, change_id).await?;
        }
        Some(Commands::Validate { path }) => {
            commands::validate::execute(path).await?;
        }
        Some(Commands::Changelog { path }) => {
            commands::changelog::execute(path).await?;
        }
        Some(Commands::Hooks {
            path,
            list,
            install,
        }) => {
            commands::hooks::execute(path, list, install).await?;
        }
        None => {
            println!("SDD AI Development Harness");
            println!("==========================");
            println!("\nUsage: sdd <command> [options]");
            println!("\nCommands:");
            println!("  init          Initialize a new SDD project");
            println!("  status        Show project status");
            println!("  brainstorm    Start a brainstorming session");
            println!("  requirements  Manage project requirements");
            println!("  questions     Manage project questions");
            println!("  architecture  Manage project architecture");
            println!("  design        Manage technical design");
            println!("  tasks         Manage project tasks");
            println!("  context       Show context for a task");
            println!("  implement     Implement a task");
            println!("  test          Test a task");
            println!("  review        Review a task");
            println!("  change        Show change details");
            println!("  impact        Analyze impact of a change");
            println!("  validate      Validate the project");
            println!("  changelog     Show project changelog");
            println!("  hooks         Manage project hooks");
            println!("\nRun 'sdd <command> --help' for more information on a command.");
        }
    }

    Ok(())
}
