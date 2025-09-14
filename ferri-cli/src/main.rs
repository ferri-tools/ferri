use clap::{Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Ferri project
    Init,
    /// Manage the context of the conversation
    Ctx {
        #[command(subcommand)]
        action: CtxCommand,
    },
    /// Execute a command with the current context
    With {
        /// The command to execute
        #[arg(required = true, trailing_var_arg = true)]
        command: Vec<String>,
    },
}

#[derive(Subcommand)]
enum CtxCommand {
    /// Add a file or directory to the context
    Add {
        /// The path to the file or directory
        path: String,
    },
    /// List the current context
    List,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            // Get the current directory
            let current_path = env::current_dir().expect("Failed to get current directory");

            // Call the core logic
            match ferri_core::initialize_project(&current_path) {
                Ok(_) => println!("Successfully initialized Ferri project in ./.ferri"),
                Err(e) => eprintln!("Error: Failed to initialize project - {}", e),
            }
        }
        Commands::Ctx { action } => match action {
            CtxCommand::Add { path } => {
                println!("unimplemented: ctx add {}", path);
            }
            CtxCommand::List => {
                println!("unimplemented: ctx list");
            }
        },
        Commands::With { command } => {
            println!("unimplemented: with {:?}", command);
        }
    }
}
