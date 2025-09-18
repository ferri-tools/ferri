use clap::{Args, Parser, Subcommand};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

mod flow_run_tui;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct SharedArgs {
    /// The model to use for the command
    #[arg(long)]
    model: Option<String>,
    /// Inject context into the command
    #[arg(long)]
    ctx: bool,
    /// The file path to save the output to
    #[arg(long)]
    output: Option<PathBuf>,
    /// The command to execute
    #[arg(required = true, trailing_var_arg = true)]
    command: Vec<String>,
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
        #[command(flatten)]
        args: SharedArgs,
    },
    /// Run a command as a background job
    Run {
        #[command(flatten)]
        args: SharedArgs,
    },
    /// List running and completed jobs
    Ps,
    /// Terminate a running job
    Kill {
        /// The ID of the job to terminate
        job_id: String,
    },
    /// Retrieve the output of a completed job
    Yank {
        /// The ID of the job to retrieve
        job_id: String,
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
    /// Manage and execute multi-step AI workflows
    Flow {
        #[command(subcommand)]
        action: FlowCommand,
    },
}

#[derive(Subcommand)]
enum FlowCommand {
    /// Run a workflow from a file
    Run {
        /// The path to the workflow file
        file: String,
    },
    /// Display a visual representation of a workflow
    Show {
        /// The path to the workflow file
        file: String,
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
    #[clap(alias = "list")]
    Ls,
    /// Remove one or more files/directories from the context
    Rm {
        /// The paths to the files or directories to remove
        #[arg(required = true, num_args = 1..)]
        paths: Vec<String>,
    },
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
    /// Remove a secret
    Rm {
        /// The name of the secret to remove
        key: String,
    },
    /// List all secret keys
    Ls,
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
        /// The Google Cloud Project ID (if required)
        #[arg(long)]
        project_id: Option<String>,
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

    // Get the current directory once for all commands that need it.
    let current_path_result = env::current_dir();

    // Handle commands that don't require initialization first.
    if let Commands::Init = &cli.command {
        let current_path = current_path_result.expect("Failed to get current directory");
        match ferri_core::initialize_project(&current_path) {
            Ok(_) => println!("Successfully initialized Ferri project in ./.ferri"),
            Err(e) => {
                eprintln!("Error: Failed to initialize project - {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // For all other commands, ensure the project is initialized.
    let current_path = match current_path_result {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: Failed to get current directory - {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = ferri_core::verify_project_initialized(&current_path) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Proceed with the command logic.
    match &cli.command {
        Commands::Init => { /* This case is handled above */ }
        Commands::Ctx { action } => match action {
            CtxCommand::Add { paths } => {
                let path_bufs = paths.iter().map(PathBuf::from).collect();
                match ferri_core::context::add_to_context(&current_path, path_bufs) {
                    Ok(_) => println!("Successfully added {} path(s) to context.", paths.len()),
                    Err(e) => eprintln!("Error: Failed to add to context - {}", e),
                }
            }
            CtxCommand::Ls => {
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
            CtxCommand::Rm { paths } => {
                let path_bufs = paths.iter().map(PathBuf::from).collect();
                match ferri_core::context::remove_from_context(&current_path, path_bufs) {
                    Ok(_) => println!("Successfully removed {} path(s) from context.", paths.len()),
                    Err(e) => eprintln!("Error: Failed to remove from context - {}", e),
                }
            }
        },
        Commands::With { args } => {
            let exec_args = ferri_core::execute::ExecutionArgs {
                model: args.model.clone(),
                use_context: args.ctx,
                output_file: args.output.clone(),
                command_with_args: args.command.clone(),
            };

            match ferri_core::execute::prepare_command(&current_path, &exec_args) {
                Ok((prepared_command, secrets)) => {
                    match prepared_command {
                        ferri_core::execute::PreparedCommand::Local(mut command) => {
                            let status = command
                                .envs(secrets)
                                .stdout(std::process::Stdio::inherit())
                                .stderr(std::process::Stdio::inherit())
                                .spawn()
                                .and_then(|mut child| child.wait());

                            if let Err(e) = status {
                                eprintln!("Error: Command execution failed - {}", e);
                                std::process::exit(1);
                            }
                        }
                        ferri_core::execute::PreparedCommand::Remote(request) => {
                            match request.send() {
                                Ok(response) => {
                                    let status = response.status();
                                    let body = response.text().unwrap_or_default();
                                    if status.is_success() {
                                        let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&body);
                                        if let Ok(json_array) = parsed {
                                            let mut full_text = String::new();
                                            for candidate in json_array {
                                                if let Some(parts) = candidate.get("candidates")
                                                    .and_then(|c| c.get(0))
                                                    .and_then(|c| c.get("content"))
                                                    .and_then(|c| c.get("parts"))
                                                    .and_then(|p| p.as_array())
                                                {
                                                    for part in parts {
                                                        // Handle text content
                                                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                                            full_text.push_str(text);
                                                        }

                                                        // Handle image content
                                                        if let (Some(output_path), Some(inline_data)) = (&exec_args.output_file, part.get("inlineData")) {
                                                            if let Some(b64_data) = inline_data.get("data").and_then(|d| d.as_str()) {
                                                                match ferri_core::execute::save_base64_image(output_path, b64_data) {
                                                                    Ok(_) => println!("Successfully saved image to {}", output_path.display()),
                                                                    Err(e) => eprintln!("Error: Failed to save image - {}", e),
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if !full_text.is_empty() {
                                                println!("{}", full_text);
                                            } else if exec_args.output_file.is_none() {
                                                eprintln!("Error: Could not extract text or image data from API response.");
                                                eprintln!("Full response: {}", body);
                                                std::process::exit(1);
                                            }
                                        } else {
                                            eprintln!("Error: Failed to parse API response as a JSON array.");
                                            eprintln!("Full response: {}", body);
                                            std::process::exit(1);
                                        }
                                    } else {
                                        eprintln!("Error: API request failed with status: {}", status);
                                        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&body);
                                        if let Ok(json) = parsed {
                                            if let Some(msg) = json["error"]["message"].as_str() {
                                                eprintln!("Details: {}", msg);
                                            } else {
                                                eprintln!("Full response: {}", body);
                                            }
                                        } else {
                                            eprintln!("Full response: {}", body);
                                        }
                                        std::process::exit(1);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error: Failed to send API request - {}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: Failed to prepare command - {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Secrets { action } => match action {
            SecretsCommand::Set { key, value } => {
                match ferri_core::secrets::set_secret(&current_path, key, value) {
                    Ok(_) => println!("Secret '{}' set successfully.", key),
                    Err(e) => eprintln!("Error: Failed to set secret - {}", e),
                }
            }
            SecretsCommand::Rm { key } => {
                match ferri_core::secrets::remove_secret(&current_path, key) {
                    Ok(_) => println!("Secret '{}' removed successfully.", key),
                    Err(e) => eprintln!("Error: Failed to remove secret - {}", e),
                }
            }
            SecretsCommand::Ls => {
                match ferri_core::secrets::list_secrets(&current_path) {
                    Ok(keys) => {
                        if keys.is_empty() {
                            println!("No secrets found.");
                        } else {
                            println!("Available secrets:");
                            for key in keys {
                                println!("- {}", key);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error: Failed to list secrets - {}", e),
                }
            }
        },
        Commands::Models { action } => match action {
            ModelsCommand::Add { alias, provider, model_name, api_key_secret, project_id } => {
                let model = ferri_core::models::Model {
                    alias: alias.clone(),
                    provider: provider.clone(),
                    model_name: model_name.clone(),
                    api_key_secret: api_key_secret.clone(),
                    project_id: project_id.clone(),
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
                print!("Are you sure you want to remove model '{}'? [y/N] ", alias);
                io::stdout().flush().unwrap();
                let mut confirmation = String::new();
                io::stdin().read_line(&mut confirmation).unwrap();

                if confirmation.trim().eq_ignore_ascii_case("y") {
                    match ferri_core::models::remove_model(&current_path, alias) {
                        Ok(_) => println!("Model '{}' removed successfully.", alias),
                        Err(e) => eprintln!("Error: Failed to remove model - {}", e),
                    }
                } else {
                    println!("Removal cancelled.");
                }
            }
        },
        Commands::Run { args } => {
            let exec_args = ferri_core::execute::ExecutionArgs {
                model: args.model.clone(),
                use_context: args.ctx,
                output_file: args.output.clone(),
                command_with_args: args.command.clone(),
            };

            match ferri_core::execute::prepare_command(&current_path, &exec_args) {
                Ok((prepared_command, secrets)) => {
                    match prepared_command {
                        ferri_core::execute::PreparedCommand::Local(command) => {
                            let mut original_command_parts = Vec::new();
                            if let Some(model) = &args.model {
                                original_command_parts.push(format!("--model {}", model));
                            }
                            if args.ctx {
                                original_command_parts.push("--ctx".to_string());
                            }
                            original_command_parts.push("--".to_string());
                            original_command_parts.extend(args.command.iter().cloned());

                            match ferri_core::jobs::submit_job(
                                &current_path,
                                command,
                                secrets,
                                &original_command_parts,
                            ) {
                                Ok(job) => {
                                    println!("Successfully submitted job '{}'.", job.id);
                                    if let Some(pid) = job.pid {
                                        println!("Process ID: {}", pid);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error: Failed to submit job - {}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        ferri_core::execute::PreparedCommand::Remote(_) => {
                            eprintln!("Error: Remote model execution cannot be run as a background job yet.");
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: Failed to prepare command - {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Ps => {
            match ferri_core::jobs::list_jobs(&current_path) {
                Ok(jobs) => {
                    if jobs.is_empty() {
                        println!("No jobs found.");
                    } else {
                        println!("{:<15} {:<15} {:<15} {:<10} {}", "JOB ID", "PID", "PGID", "STATUS", "COMMAND");
                        for job in jobs {
                            let pid_str = job.pid.map_or("N/A".to_string(), |p| p.to_string());
                            let pgid_str = job.pgid.map_or("N/A".to_string(), |p| p.to_string());
                            println!("{:<15} {:<15} {:<15} {:<10} {}", job.id, pid_str, pgid_str, job.status, job.command);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: Failed to list jobs - {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Kill { job_id } => {
            match ferri_core::jobs::kill_job(&current_path, job_id) {
                Ok(_) => println!("Successfully terminated job '{}'.", job_id),
                Err(e) => {
                    eprintln!("Error: Failed to terminate job - {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Yank { job_id } => {
            match ferri_core::jobs::get_job_output(&current_path, job_id) {
                Ok(output) => {
                    print!("{}", output);
                }
                Err(e) => {
                    eprintln!("Error: Failed to get job output - {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Flow { action } => match action {
            FlowCommand::Run { file } => {
                let file_path = PathBuf::from(file);
                match ferri_core::flow::parse_pipeline_file(&file_path) {
                    Ok(pipeline) => {
                        // Launch the new real-time TUI
                        if let Err(e) = flow_run_tui::run(pipeline) {
                             eprintln!("Error: Flow execution failed - {}", e);
                             std::process::exit(1);
                        }
                        // The old direct execution is now handled by the TUI
                    }
                    Err(e) => {
                        eprintln!("Error: Failed to parse flow file - {}", e);
                        std::process::exit(1);
                    }
                }
            }
            FlowCommand::Show { file } => {
                let file_path = PathBuf::from(file);
                match ferri_core::flow::parse_pipeline_file(&file_path) {
                    Ok(pipeline) => {
                        if let Err(e) = ferri_core::flow::show_pipeline(&pipeline) {
                            eprintln!("Error: Flow visualization failed - {}", e);
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: Failed to parse flow file - {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },
    }
}
