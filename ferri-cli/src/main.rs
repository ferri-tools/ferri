use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;

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
    /// Manage encrypted secrets
    Secrets {
        #[command(subcommand)]
        action: SecretsCommand,
    },
    /// Manage AI models
    Models {
        #[command(subcommand)]
        action: ModelsCommand,
    },
}

#[derive(Subcommand)]
enum CtxCommand {
    /// Add one or more files/directories to the context
    Add {
        /// The paths to the files or directories
        #[arg(required = true, num_args = 1..)]
        paths: Vec<String>,
    },
    /// List the current context
    List,
}

#[derive(Subcommand)]
enum SecretsCommand {
    /// Set a secret
    Set {
        /// The name of the secret
        key: String,
        /// The value of the secret
        value: String,
    },
}

#[derive(Subcommand)]
enum ModelsCommand {
    /// Add a new model to the registry
    Add {
        /// A short, memorable alias for the model
        alias: String,
        /// The provider (e.g., 'ollama', 'openai', 'google')
        #[arg(long)]
        provider: String,
        /// The actual model name used by the provider (e.g., 'llama3:latest', 'gpt-4o')
        #[arg(long)]
        model_name: String,
        /// The name of the secret holding the API key (if required)
        #[arg(long)]
        api_key_secret: Option<String>,
    },
    /// List all available models
    Ls,
    /// Remove a model from the registry
    Rm {
        /// The alias of the model to remove
        alias: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            let current_path = env::current_dir().expect("Failed to get current directory");
            match ferri_core::initialize_project(&current_path) {
                Ok(_) => println!("Successfully initialized Ferri project in ./.ferri"),
                Err(e) => eprintln!("Error: Failed to initialize project - {}", e),
            }
        }
        Commands::Ctx { action } => {
            let current_path = env::current_dir().expect("Failed to get current directory");
            match action {
                CtxCommand::Add { paths } => {
                    let path_bufs = paths.iter().map(PathBuf::from).collect();
                    match ferri_core::context::add_to_context(&current_path, path_bufs) {
                        Ok(_) => println!("Successfully added {} path(s) to context.", paths.len()),
                        Err(e) => eprintln!("Error: Failed to add to context - {}", e),
                    }
                }
                CtxCommand::List => {
                    match ferri_core::context::list_context(&current_path) {
                        Ok(files) => {
                            if files.is_empty() {
                                println!("Context is empty.");
                            } else {
                                println!("Current context:");
                                for file in files {
                                    println!("- {}", file);
                                }
                            }
                        }
                        Err(e) => eprintln!("Error: Failed to list context - {}", e),
                    }
                }
            }
        },
        Commands::With { command } => {
            println!("unimplemented: with {:?}", command);
        }
        Commands::Secrets { action } => match action {
            SecretsCommand::Set { key, value } => {
                let current_path = env::current_dir().expect("Failed to get current directory");
                match ferri_core::secrets::set_secret(&current_path, key, value) {
                    Ok(_) => println!("Secret '{}' set successfully.", key),
                    Err(e) => eprintln!("Error: Failed to set secret - {}", e),
                }
            }
        },
        Commands::Models { action } => {
            let current_path = env::current_dir().expect("Failed to get current directory");
            match action {
                ModelsCommand::Add { alias, provider, model_name, api_key_secret } => {
                    let model = ferri_core::models::Model {
                        alias: alias.clone(),
                        provider: provider.clone(),
                        model_name: model_name.clone(),
                        api_key_secret: api_key_secret.clone(),
                        discovered: false,
                    };
                    match ferri_core::models::add_model(&current_path, model) {
                        Ok(_) => println!("Model '{}' added successfully.", alias),
                        Err(e) => eprintln!("Error: Failed to add model - {}", e),
                    }
                }
                ModelsCommand::Ls => {
                    match ferri_core::models::list_models(&current_path) {
                        Ok(models) => {
                            println!("{:<20} {:<15} {:<30} {:<15}", "ALIAS", "PROVIDER", "ID/NAME", "TYPE");
                            for model in models {
                                let model_type = if model.discovered { "(discovered)" } else { "" };
                                println!("{:<20} {:<15} {:<30} {:<15}", model.alias, model.provider, model.model_name, model_type);
                            }
                        }
                        Err(e) => eprintln!("Error: Failed to list models - {}", e),
                    }
                }
                ModelsCommand::Rm { alias } => {
                    match ferri_core::models::remove_model(&current_path, alias) {
                        Ok(_) => println!("Model '{}' removed successfully.", alias),
                        Err(e) => eprintln!("Error: Failed to remove model - {}", e),
                    }
                }
            }
        },
    }
}
