use anyhow::{anyhow, bail, Result};
use clap::Parser;
use colored::*;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Data Structures for Plan and Ollama API ---
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Step {
    id: String,
    command: String,
    dependencies: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Plan {
    goal: String,
    steps: Vec<Step>,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    system: String,
    // Add the format field, which tells Ollama to guarantee JSON output.
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

// --- The Live AI Planner ---
struct OllamaPlanner;

impl OllamaPlanner {
    async fn generate_plan(prompt: &str, model: &str) -> Result<Plan> {
        println!("\n{} {}...", "🤔 Thinking with".yellow(), model.bold());

        // --- SIMPLIFIED FEW-SHOT PROMPT ---
        // We no longer need to explicitly tell the model to output JSON,
        // as Ollama's `format` parameter will enforce it. We just need to guide the content.
        let system_prompt = "You are an expert software developer and terminal assistant. Your goal is to break down a user's high-level request into a precise, executable plan.
        
The plan's structure should have a 'goal' and a list of 'steps'. Each step needs an 'id', a 'command', and a list of 'dependencies'.

--- EXAMPLE ---
USER PROMPT: \"create a new react component named Header\"
YOUR RESPONSE:
{
  \"goal\": \"create a new react component named Header\",
  \"steps\": [
    {
      \"id\": \"create_folder\",
      \"command\": \"mkdir -p src/components/Header\",
      \"dependencies\": []
    },
    {
      \"id\": \"create_jsx\",
      \"command\": \"touch src/components/Header/Header.jsx\",
      \"dependencies\": [\"create_folder\"]
    },
    {
      \"id\": \"create_css\",
      \"command\": \"touch src/components/Header/Header.css\",
      \"dependencies\": [\"create_folder\"]
    }
  ]
}".to_string();

        let client = reqwest::Client::new();
        // --- UPDATE PAYLOAD ---
        // Add the format parameter to the request.
        let request_payload = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            system: system_prompt,
            format: Some("json".to_string()),
        };

        let res = client
            .post("http://localhost:11434/api/generate")
            .json(&request_payload)
            .send()
            .await?;

        if !res.status().is_success() {
            bail!("Ollama API request failed with status: {}", res.status());
        }

        let ollama_response = res.json::<OllamaResponse>().await?;
        
        // The cleaner is now less critical but still good practice.
        let cleaned_response = extract_json_from_response(&ollama_response.response);

        let plan: Plan = serde_json::from_str(cleaned_response)
            .map_err(|e| anyhow!("Failed to parse LLM response into a valid plan. Error: {}. \n--- Cleaned Response ---\n{}", e, cleaned_response))?;
            
        Ok(plan)
    }
}

fn extract_json_from_response(raw_text: &str) -> &str {
    let trimmed = raw_text.trim();
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if start < end {
            return &trimmed[start..=end];
        }
    }
    trimmed
}


// --- CLI Definition using clap ---
#[derive(Parser, Debug)]
#[command(author, version, about = "A prototype for the `ferri do` agentic command.", long_about = None)]
struct Cli {
    #[arg(required = true, num_args = 1..)]
    prompt: Vec<String>,
    #[arg(short, long, default_value = "gemma:2b")]
    model: String,
}

// --- Main Application Logic (now async) ---
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let user_prompt = cli.prompt.join(" ");

    println!("{}\n", "🤖 Ferri Agent Activated".bold().cyan());
    println!("{} {}", "🎯 Goal:".bold().white(), user_prompt.italic());

    let plan = OllamaPlanner::generate_plan(&user_prompt, &cli.model).await?;

    println!("\n{}\n", "📋 Proposed Plan:".bold().white());
    for (index, step) in plan.steps.iter().enumerate() {
        println!(
            "  {}. {} ({})",
            (index + 1).to_string().bold(),
            step.command.green(),
            step.id.cyan()
        );
    }

    let mut graph = DiGraph::<String, ()>::new();
    let mut node_map = HashMap::<String, NodeIndex>::new();
    for step in &plan.steps {
        let node = graph.add_node(step.id.clone());
        node_map.insert(step.id.clone(), node);
    }
    for step in &plan.steps {
        let to_node = node_map[&step.id];
        for dep_id in &step.dependencies {
            if let Some(&from_node) = node_map.get(dep_id) {
                graph.add_edge(from_node, to_node, ());
            }
        }
    }

    println!("\n{}\n", "📈 Execution Graph (DAG):".bold().white());
    let roots: Vec<NodeIndex> = graph.externals(petgraph::Direction::Incoming).collect();
    for root in roots {
        println!("{} {}", "▶".cyan(), graph[root].clone().bold().magenta());
        print_dag_branch(&graph, root, String::new(), true);
    }

    println!("\n{}\n", "Confirm execution? [y/N]".bold().yellow());
    Ok(())
}

fn print_dag_branch(
    graph: &DiGraph<String, ()>,
    node: NodeIndex,
    prefix: String,
    _is_last_parent_branch: bool,
) {
    let children: Vec<_> = graph.neighbors(node).collect();
    let children_count = children.len();
    for (i, child) in children.iter().enumerate() {
        let is_last_child = i == children_count - 1;
        let connector = if is_last_child { "└─" } else { "├─" };
        let node_label = graph[*child].clone();
        println!(
            "{}{} {} {}",
            prefix,
            connector.cyan(),
            "●".yellow(),
            node_label.bold().magenta()
        );
        let new_prefix = prefix.clone() + if is_last_child { "   " } else { "│  " };
        print_dag_branch(graph, *child, new_prefix, is_last_child);
    }
}

