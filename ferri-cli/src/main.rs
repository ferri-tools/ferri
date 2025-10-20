use clap::{Parser, Subcommand};
use colored::*;
use ferri_automation::execute::{self, SharedArgs};
use ferri_automation::{flow, jobs};
use ferri_core::{context, models, project, secrets};
use rand::Rng;
use serde_json::json;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

// These modules are part of the CLI binary, not library crates.
mod agent_tui;
mod flow_run_tui;
mod ps_tui;
mod tui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Ctx {
        #[command(subcommand)]
        action: CtxCommand,
    },
    With {
        #[command(flatten)]
        args: SharedArgs,
    },
    Run {
        #[command(flatten)]
        args: SharedArgs,
    },
    Ps,
    Kill {
        job_id: String,
    },
    Yank {
        job_id: String,
    },
    Secrets {
        #[command(subcommand)]
        action: SecretsCommand,
    },
    Models {
        #[command(subcommand)]
        action: ModelsCommand,
    },
    Flow {
        #[command(subcommand)]
        action: FlowCommand,
    },
    Do {
        #[arg(required = true, trailing_var_arg = true)]
        prompt: Vec<String>,
    },
    Ui,
    #[command(hide = true)]
    Doctor,
    #[command(hide = true)]
    Runtime {
        #[command(subcommand)]
        action: RuntimeCommand,
    },
}

#[derive(Subcommand)]
enum RuntimeCommand {
    SetOutput {
        name: String,
        value: String,
    },
}

#[derive(Subcommand)]
enum FlowCommand {
    Run { file: String },
    Show { file: String },
}

#[derive(Subcommand)]
enum CtxCommand {
    Add {
        #[arg(required = true, num_args = 1..)]
        paths: Vec<String>,
    },
    #[clap(alias = "list")]
    Ls,
    Rm {
        #[arg(required = true, num_args = 1..)]
        paths: Vec<String>,
    },
    Clear,
}

#[derive(Subcommand)]
enum SecretsCommand {
    Set {
        key: String,
        value: Option<String>,
    },
    Rm {
        key: String,
    },
    Ls,
}

#[derive(Subcommand)]
enum ModelsCommand {
    Add {
        alias: String,
        #[arg(long)]
        provider: String,
        #[arg(long)]
        model_name: String,
        #[arg(long)]
        api_key_secret: Option<String>,
    },
    Ls,
    Rm {
        alias: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let current_path_result = env::current_dir();

    if let Commands::Init = &cli.command {
        let current_path = current_path_result.expect("Failed to get current directory");
        match project::initialize_project(&current_path) {
            Ok(_) => print_init_message(),
            Err(e) => {
                eprintln!("Error: Failed to initialize project - {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    let current_path = match current_path_result {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: Failed to get current directory - {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = project::verify_project_initialized(&current_path) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    match &cli.command {
        Commands::Init => {} // This case is handled above
        Commands::Ctx { action } => match action {
            CtxCommand::Add { paths } => {
                let path_bufs = paths.iter().map(PathBuf::from).collect();
                match context::add_to_context(&current_path, path_bufs) {
                    Ok(_) => println!("Successfully added {} path(s) to context.", paths.len()),
                    Err(e) => eprintln!("Error: Failed to add to context - {}", e),
                }
            }
            CtxCommand::Ls => match context::list_context(&current_path) {
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
            },
            CtxCommand::Rm { paths } => {
                let path_bufs = paths.iter().map(PathBuf::from).collect();
                match context::remove_from_context(&current_path, path_bufs) {
                    Ok(_) => println!("Successfully removed {} path(s) from context.", paths.len()),
                    Err(e) => eprintln!("Error: Failed to remove from context - {}", e),
                }
            }
            CtxCommand::Clear => match context::clear_context(&current_path) {
                Ok(_) => println!("Successfully cleared context."),
                Err(e) => eprintln!("Error: Failed to clear context - {}", e),
            },
        },
        Commands::With { args } => {
            // TODO: Streaming is temporarily disabled due to a bug. Re-enable when fixed.
            // The previous implementation used a separate path for streaming Google models.
            // This has been consolidated into a single, non-streaming path for all models.
            let exec_args = execute::ExecutionArgs {
                model: args.model.clone(),
                use_context: args.ctx,
                output_file: args.output.clone(),
                command_with_args: args.command.clone(),
                streaming: false,
            };

            if args.ctx {
                if let Ok(files) = context::list_context(&current_path) {
                    if files.is_empty() {
                        eprintln!("Warning: --ctx flag was used, but the context is empty.");
                        eprintln!("You can add files to the context with `ferri ctx add <paths...>`");
                    }
                }
            }

            match execute::prepare_command(&current_path, &exec_args) {
                Ok((prepared_command, secrets)) => match prepared_command {
                    execute::PreparedCommand::Local(mut command, stdin_data) => {
                        // Inject secrets directly into the command's environment
                        for (key, value) in &secrets {
                            command.env(key, value);
                        }

                        if stdin_data.is_some() {
                            command.stdin(std::process::Stdio::piped());
                        }
                        command.stdout(std::process::Stdio::piped());
                        command.stderr(std::process::Stdio::inherit());

                        match command.spawn() {
                            Ok(mut child) => {
                                if let Some(data) = stdin_data {
                                    if let Some(mut stdin) = child.stdin.take() {
                                        if let Err(e) = stdin.write_all(data.as_bytes()) {
                                            eprintln!("Error: Failed to write to command stdin - {}", e);
                                            std::process::exit(1);
                                        }
                                    }
                                }

                                match child.wait_with_output() {
                                    Ok(output) => {
                                        if output.status.success() {
                                            if let Ok(stdout_str) = String::from_utf8(output.stdout) {
                                                if let Some(output_path) = &exec_args.output_file {
                                                    if let Err(e) = fs::write(output_path, &stdout_str) {
                                                        eprintln!("Error: Failed to write to output file {} - {}", output_path.display(), e);
                                                        std::process::exit(1);
                                                    }
                                                } else {
                                                    print!("{}", stdout_str);
                                                }
                                            } else {
                                                eprintln!("Error: Failed to decode command output as UTF-8.");
                                            }
                                        } else {
                                            eprintln!("Error: Command execution failed with status: {}", output.status);
                                            std::process::exit(1);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error: Failed to wait for command output - {}", e);
                                        std::process::exit(1);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: Failed to spawn command - {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    execute::PreparedCommand::Remote(request) => match request.send() {
                        Ok(response) => {
                            let status = response.status();
                            let body = response.text().unwrap_or_default();
                            if status.is_success() {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                                    let mut text_content = String::new();
                                    let mut image_saved = false;
                                    let response_chunks = if let Some(array) = json.as_array() { array.to_vec() } else { vec![json] };
                                    for chunk in response_chunks {
                                        if let Some(candidates) = chunk.get("candidates").and_then(|c| c.as_array()) {
                                            for candidate in candidates {
                                                if let Some(parts) = candidate.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                                                    for part in parts {
                                                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                                            text_content.push_str(text);
                                                        }
                                                        if let (Some(output_path), Some(inline_data)) = (&exec_args.output_file, part.get("inlineData")) {
                                                            if let Some(b64_data) = inline_data.get("data").and_then(|d| d.as_str()) {
                                                                match execute::save_base64_image(output_path, b64_data) {
                                                                    Ok(_) => {
                                                                        println!("Successfully saved image to {}", output_path.display());
                                                                        image_saved = true;
                                                                    }
                                                                    Err(e) => eprintln!("Error: Failed to save image - {}", e),
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if !text_content.is_empty() {
                                        if let Some(output_path) = &exec_args.output_file {
                                            if let Err(e) = fs::write(output_path, &text_content) {
                                                eprintln!("Error: Failed to write to output file {} - {}", output_path.display(), e);
                                                std::process::exit(1);
                                            }
                                        } else {
                                            print!("{}", text_content);
                                        }
                                    } else if !image_saved {
                                        eprintln!("Error: Could not extract text or image data from API response.");
                                        eprintln!("Full response: {}", body);
                                        std::process::exit(1);
                                    }
                                } else {
                                    eprintln!("Error: Failed to parse API response as JSON.");
                                    eprintln!("Full response: {}", body);
                                    std::process::exit(1);
                                }
                            } else {
                                eprintln!("Error: API request failed with status: {}", status);
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                                    if let Some(msg) = json.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
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
                    },
                },
                Err(e) => {
                    eprintln!("Error: Failed to prepare command - {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Secrets { action } => match action {
            SecretsCommand::Set { key, value } => {
                if let Err(e) = secrets::set_secret(&current_path, key, value.clone()) {
                    eprintln!("Error: Failed to set secret - {}", e)
                }
            }
            SecretsCommand::Rm { key } => match secrets::remove_secret(&current_path, key) {
                Ok(_) => println!("Secret '{}' removed successfully.", key),
                Err(e) => eprintln!("Error: Failed to remove secret - {}", e),
            },
            SecretsCommand::Ls => match secrets::list_secrets(&current_path) {
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
            },
        },
        Commands::Models { action } => match action {
            ModelsCommand::Add { alias, provider, model_name, api_key_secret } => {
                let model = models::Model {
                    alias: alias.clone(),
                    provider: provider.clone(),
                    model_name: model_name.clone(),
                    api_key_secret: api_key_secret.clone(),
                    discovered: false,
                };
                match models::add_model(&current_path, model) {
                    Ok(_) => println!("Model '{}' added successfully.", alias),
                    Err(e) => eprintln!("Error: Failed to add model - {}", e),
                }
            }
            ModelsCommand::Ls => match models::list_models(&current_path) {
                Ok(models) => {
                    println!("{:<20} {:<15} {:<30} {:<15}", "ALIAS", "PROVIDER", "ID/NAME", "TYPE");
                    for model in models {
                        let model_type = if model.discovered { "(discovered)" } else { "" };
                        println!("{:<20} {:<15} {:<30} {:<15}", model.alias, model.provider, model.model_name, model_type);
                    }
                }
                Err(e) => eprintln!("Error: Failed to list models - {}", e),
            },
            ModelsCommand::Rm { alias } => {
                print!("Are you sure you want to remove model '{}'? [y/N] ", alias);
                io::stdout().flush().unwrap();
                let mut confirmation = String::new();
                io::stdin().read_line(&mut confirmation).unwrap();
                if confirmation.trim().eq_ignore_ascii_case("y") {
                    match models::remove_model(&current_path, alias) {
                        Ok(_) => println!("Model '{}' removed successfully.", alias),
                        Err(e) => eprintln!("Error: Failed to remove model - {}", e),
                    }
                } else {
                    println!("Removal cancelled.");
                }
            }
        },
        Commands::Run { args } => {
            let exec_args = execute::ExecutionArgs {
                model: args.model.clone(),
                use_context: args.ctx,
                output_file: args.output.clone(),
                command_with_args: args.command.clone(),
                streaming: false,
            };
            match execute::prepare_command(&current_path, &exec_args) {
                Ok((prepared_command, secrets)) => {
                    let mut original_command_parts = Vec::new();
                    if let Some(model) = &args.model {
                        original_command_parts.push(format!("--model {}", model));
                    }
                    if args.ctx {
                        original_command_parts.push("--ctx".to_string());
                    }
                    original_command_parts.push("--".to_string());
                    original_command_parts.extend(args.command.iter().cloned());
                    match jobs::submit_job(&current_path, prepared_command, secrets, &original_command_parts, None, exec_args.output_file) {
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
                Err(e) => {
                    eprintln!("Error: Failed to prepare command - {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Ps => match jobs::list_jobs(&current_path) {
            Ok(jobs) => {
                if jobs.is_empty() {
                    println!("No jobs found.");
                } else {
                    println!("{:<10} {:<12} {:<30} {:<20}", "ID", "Status", "Command", "Start Time");
                    for job in jobs {
                        println!("{:<10} {:<12} {:<30} {:<20}", job.id, job.status, job.command, job.start_time.format("%Y-%m-%d %H:%M:%S"));
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: Failed to list jobs - {}", e);
                std::process::exit(1);
            }
        },
        Commands::Kill { job_id } => match jobs::kill_job(&current_path, job_id) {
            Ok(_) => println!("Successfully terminated job '{}'.", job_id),
            Err(e) => {
                eprintln!("Error: Failed to terminate job - {}", e);
                std::process::exit(1);
            }
        },
        Commands::Yank { job_id } => match jobs::get_job_output(&current_path, job_id) {
            Ok(output) => print!("{}", output),
            Err(e) => {
                eprintln!("Error: Failed to get job output - {}", e);
                std::process::exit(1);
            }
        },
        Commands::Flow { action } => match action {
            FlowCommand::Run { file } => {
                let file_path = PathBuf::from(file);

                // Try parsing as new format first
                match flow::parse_flow_file(&file_path) {
                    Ok(flow_doc) => {
                        // New format - use orchestrator
                        println!("🚀 Running flow: {} ({})", flow_doc.metadata.name, flow_doc.api_version);
                        println!("   Jobs: {}", flow_doc.spec.jobs.len());
                        println!();

                        // Execute the flow with rich output
                        if let Err(e) = execute_flow_with_output(flow_doc, &current_path) {
                            eprintln!("\n❌ Flow execution failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                    Err(new_format_error) => {
                        // Fall back to legacy format
                        match flow::parse_pipeline_file(&file_path) {
                            Ok(pipeline) => {
                                if let Err(e) = flow_run_tui::run(pipeline) {
                                    eprintln!("Error: Flow execution failed - {}", e);
                                    std::process::exit(1);
                                }
                            }
                            Err(_legacy_error) => {
                                // Both parsers failed - show the new format error since that's preferred
                                eprintln!("Error: {}", new_format_error);
                                std::process::exit(1);
                            }
                        }
                    }
                }
            }
            FlowCommand::Show { file } => {
                let file_path = PathBuf::from(file);

                // Try parsing as new format first
                match flow::parse_flow_file(&file_path) {
                    Ok(flow_doc) => {
                        // Display new format flow info
                        println!("Flow: {}", flow_doc.metadata.name);
                        println!("API Version: {}", flow_doc.api_version);
                        println!("Kind: {}", flow_doc.kind);
                        println!("\nJobs:");
                        for (job_id, job) in &flow_doc.spec.jobs {
                            let name = job.name.as_ref().unwrap_or(job_id);
                            println!("  {} ({})", job_id, name);
                            println!("    Steps: {}", job.steps.len());
                            if let Some(needs) = &job.needs {
                                println!("    Depends on: {}", needs.join(", "));
                            }
                        }
                    }
                    Err(new_format_error) => {
                        // Fall back to legacy format
                        match flow::parse_pipeline_file(&file_path) {
                            Ok(pipeline) => {
                                if let Err(e) = flow::show_pipeline(&pipeline) {
                                    eprintln!("Error: Flow visualization failed - {}", e);
                                    std::process::exit(1);
                                }
                            }
                            Err(_legacy_error) => {
                                // Both parsers failed - show the new format error since that's preferred
                                eprintln!("Error: {}", new_format_error);
                                std::process::exit(1);
                            }
                        }
                    }
                }
            }
        },
        Commands::Do { prompt } => {
            let prompt_str = prompt.join(" ");
            if let Err(e) = agent_tui::run(&prompt_str) {
                eprintln!("Error: Agent TUI failed - {}", e);
                std::process::exit(1);
            }
        }
        Commands::Ui => {
            if let Err(e) = tui::runner::run_tui() {
                eprintln!("Error: Failed to launch UI - {}", e);
                std::process::exit(1);
            }
        }
        Commands::Doctor => {
            println!("-- Ferri Doctor --");
            println!("Running diagnostics...");
            print!("1. Checking for 'ollama' executable... ");
            match std::process::Command::new("which").arg("ollama").output() {
                Ok(output) if output.status.success() => {
                    println!("OK (Found at {})", String::from_utf8_lossy(&output.stdout).trim());
                }
                _ => {
                    println!("FAIL");
                    eprintln!("   Error: 'ollama' command not found in your system's PATH.");
                    eprintln!("   Please install Ollama from https://ollama.com and ensure it's accessible.");
                    std::process::exit(1);
                }
            }
            print!("2. Checking 'ollama' service status... ");
            match std::process::Command::new("ollama").arg("ps").output() {
                Ok(output) if output.status.success() => println!("OK (Service is responsive)"),
                Ok(output) => {
                    println!("FAIL");
                    eprintln!("   Error: The 'ollama' service appears to be down.");
                    eprintln!("   Please start the Ollama application and try again.");
                    eprintln!("   Details: {}", String::from_utf8_lossy(&output.stderr));
                    std::process::exit(1);
                }
                Err(e) => {
                    println!("FAIL");
                    eprintln!("   Error: Failed to execute 'ollama ps'.");
                    eprintln!("   Details: {}", e);
                    std::process::exit(1);
                }
            }
            print!("3. Checking '.ferri' directory permissions... ");
            let ferri_dir = current_path.join(".ferri");
            match std::fs::metadata(&ferri_dir) {
                Ok(metadata) => {
                    if metadata.permissions().readonly() {
                        println!("FAIL");
                        eprintln!("   Error: The '.ferri' directory is read-only.");
                    } else {
                        println!("OK");
                    }
                }
                Err(_) => {
                    println!("FAIL");
                    eprintln!("   Error: The '.ferri' directory does not exist or cannot be accessed.");
                    eprintln!("   Please run 'ferri init' first.");
                }
            }
            println!("\n--- Diagnostics Complete ---");
        }
        Commands::Runtime { action } => match action {
            RuntimeCommand::SetOutput { name, value } => {
                // Read the output file path from environment variable
                let output_file = match env::var("FERRI_OUTPUT_FILE") {
                    Ok(path) => path,
                    Err(_) => {
                        eprintln!("Error: FERRI_OUTPUT_FILE environment variable not set");
                        eprintln!("This command should only be called from within a ferri flow step");
                        std::process::exit(1);
                    }
                };

                // Append the output to the file in the format: name=value
                let output_line = format!("{}={}\n", name, value);
                match fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&output_file)
                {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(output_line.as_bytes()) {
                            eprintln!("Error: Failed to write to output file - {}", e);
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: Failed to open output file '{}' - {}", output_file, e);
                        std::process::exit(1);
                    }
                }
            }
        },
    }
}

fn execute_flow_with_output(
    flow_doc: ferri_automation::flow::FlowDocument,
    base_path: &PathBuf,
) -> io::Result<()> {
    
    use ferri_automation::orchestrator::FlowOrchestrator;
    use std::collections::HashMap;

    // Create orchestrator
    let orchestrator = FlowOrchestrator::new(
        flow_doc,
        base_path,
        HashMap::new(), // TODO: Parse runtime inputs from CLI args
    );

    // Spawn execution thread
    let execution_handle = std::thread::spawn(move || orchestrator.execute());

    // Wait for execution to complete
    match execution_handle.join() {
        Ok(result) => {
            result?;
            println!("\n✨ Flow completed successfully!");
            Ok(())
        }
        Err(_) => {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Flow execution thread panicked"
            ))
        }
    }
}

fn print_init_message() {
    // A thread-local random number generator
    let mut rng = rand::thread_rng();

    // Define our "rusty, shiny" color palette using function pointers.
    // This is an array of functions that take a &str and return a ColoredString.
    let palette: &[fn(&str) -> ColoredString] = &[
        |s| s.red(),
        |s| s.bright_red(),
        |s| s.yellow(),
        |s| s.bright_yellow().bold(),  // Shiny!
        |s| s.white().bold(),          // Shiny!
        |s| s.cyan(),                  // For a bit of patina/verdigris
        |s| s.truecolor(184, 115, 51), // A nice bronze/rust color
    ];

    let replacement_alphabet = ['+', '*', '='];

    let art = r#"
    ███████ ███████ ██████  ██████  ██
    ██      ██      ██   ██ ██   ██ ██
    █████   █████   ██████  ██████  ██
    ██      ██      ██   ██ ██   ██ ██
    ██      ███████ ██   ██ ██   ██ ██
                                                     **
                                                     **
                                                     **
                                                 ++++**++++
                                                ***********+
                                   *************************************+
                                  ***                                 =***
                                 ***           =+=       -=+-          =**=
                                +**          ***=*=      **+***         -***
                               ***********  ***=-  *=  **   +**=  **********=
                              =********** =+ ***=**********+**+ *  +*********=
                             +*********+   ==***************+*+==   **********+
                            +*********=  **+**++****=*+=***++++* ++  **********+
                           +**********    **+*+= **+ +* **+ =** *=    +*********
                         *==**+=    -        * +** ****** ** =*        =    =+** *+
                    =****** **************************************************** ******=
                 +********* ***************************************************+ *********
                  -******** +************=***+***+**********+**+**=************==********
                   ********+=*************+++==+==+==+==+==+++++=+************+ ********+
                    ********-**************************************************+********
                    =********+************+                      +************+********
                     =******++***********                          *********** *******

                      *##***************                            *****************=
              =********************************              +******************************+
    "#;
    let mut recipe: Vec<serde_json::Value> = Vec::new();
    for c in art.chars() {
        match c {
            ' ' | '\n' | '\r' => {
                // ...just print it as is.
                print!("{c}");
            }
            _ => {
                let alphabet_random_index = rng.gen_range(0..replacement_alphabet.len());
                let alphabet_replacement_character = replacement_alphabet[alphabet_random_index];
                let random_index = rng.gen_range(0..palette.len());
                let color_fn = palette[random_index];
                print!("{}", color_fn(&alphabet_replacement_character.to_string()));
                recipe.push(json!({
                    "original": c,
                    "replacement": alphabet_replacement_character,
                    "color": random_index
                }));
            }
        }
    }

    let recipe_json =
        serde_json::to_string_pretty(&recipe).expect("Failed to serialize ferri tattoo");
    let signature_path = PathBuf::from(".ferri").join("signatures.json");
    fs::write(signature_path, recipe_json).expect("Failed to serialize tattoo file");

    // Ensure the cursor moves to the next line after the art is done.
    println!();
    println!("Ferri project initialized!");
    println!("Run `{}` to see what you can do.", "ferri --help".cyan());
}