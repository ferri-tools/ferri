//! Expression evaluator for ferri-flow.yml
//!
//! Supports the `${{ <expression> }}` syntax for accessing:
//! - ctx.inputs.<name>
//! - ctx.steps.<id>.outputs.<name>
//! - ctx.jobs.<id>.outputs.<name>

use std::collections::HashMap;
use std::io;
use regex::Regex;

/// Runtime context for expression evaluation
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Flow input values
    pub inputs: HashMap<String, String>,

    /// Step outputs within the current job (step_id -> output_name -> value)
    pub step_outputs: HashMap<String, HashMap<String, String>>,

    /// Job outputs from dependencies (job_id -> output_name -> value)
    pub job_outputs: HashMap<String, HashMap<String, String>>,
}

impl EvaluationContext {
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            step_outputs: HashMap::new(),
            job_outputs: HashMap::new(),
        }
    }

    pub fn with_inputs(mut self, inputs: HashMap<String, String>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn add_step_output(&mut self, step_id: String, output_name: String, value: String) {
        self.step_outputs
            .entry(step_id)
            .or_insert_with(HashMap::new)
            .insert(output_name, value);
    }

    pub fn add_job_output(&mut self, job_id: String, output_name: String, value: String) {
        self.job_outputs
            .entry(job_id)
            .or_insert_with(HashMap::new)
            .insert(output_name, value);
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluate all expressions in a string
pub fn evaluate_expressions(text: &str, ctx: &EvaluationContext) -> io::Result<String> {
    // Pattern matches ${{ any content }}
    let expr_pattern = Regex::new(r"\$\{\{\s*(.+?)\s*\}\}")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

    let mut result = text.to_string();
    let mut errors = Vec::new();

    // Find all expression matches and evaluate them
    for cap in expr_pattern.captures_iter(text) {
        if let Some(expr) = cap.get(1) {
            match evaluate_single_expression(expr.as_str(), ctx) {
                Ok(value) => {
                    let full_match = cap.get(0).unwrap().as_str();
                    result = result.replace(full_match, &value);
                }
                Err(e) => {
                    errors.push(format!("Error evaluating '{}': {}", expr.as_str(), e));
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            errors.join("; ")
        ));
    }

    Ok(result)
}

/// Evaluate a single expression (without the ${{ }} wrapper)
fn evaluate_single_expression(expr: &str, ctx: &EvaluationContext) -> io::Result<String> {
    let parts: Vec<&str> = expr.split('.').collect();

    if parts.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Empty expression"
        ));
    }

    // All expressions must start with "ctx"
    if parts[0] != "ctx" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Expression must start with 'ctx', got '{}'", parts[0])
        ));
    }

    if parts.len() < 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid expression format: {}", expr)
        ));
    }

    match parts[1] {
        "inputs" => {
            // ctx.inputs.<name>
            if parts.len() != 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid inputs expression: {}", expr)
                ));
            }
            let input_name = parts[2];
            ctx.inputs.get(input_name)
                .map(|s| s.clone())
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Input '{}' not found", input_name)
                ))
        }
        "steps" => {
            // ctx.steps.<id>.outputs.<name>
            if parts.len() != 5 || parts[3] != "outputs" {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid steps expression, expected 'ctx.steps.<id>.outputs.<name>': {}", expr)
                ));
            }
            let step_id = parts[2];
            let output_name = parts[4];

            ctx.step_outputs
                .get(step_id)
                .and_then(|outputs| outputs.get(output_name))
                .map(|s| s.clone())
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Step output '{}.{}' not found", step_id, output_name)
                ))
        }
        "jobs" => {
            // ctx.jobs.<id>.outputs.<name>
            if parts.len() != 5 || parts[3] != "outputs" {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid jobs expression, expected 'ctx.jobs.<id>.outputs.<name>': {}", expr)
                ));
            }
            let job_id = parts[2];
            let output_name = parts[4];

            ctx.job_outputs
                .get(job_id)
                .and_then(|outputs| outputs.get(output_name))
                .map(|s| s.clone())
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Job output '{}.{}' not found", job_id, output_name)
                ))
        }
        _ => {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown context type '{}', expected 'inputs', 'steps', or 'jobs'", parts[1])
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_expression() {
        let mut ctx = EvaluationContext::new();
        ctx.inputs.insert("name".to_string(), "Alice".to_string());

        let result = evaluate_expressions("Hello ${{ ctx.inputs.name }}!", &ctx).unwrap();
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_step_output_expression() {
        let mut ctx = EvaluationContext::new();
        ctx.add_step_output("step1".to_string(), "result".to_string(), "42".to_string());

        let result = evaluate_expressions("The answer is ${{ ctx.steps.step1.outputs.result }}", &ctx).unwrap();
        assert_eq!(result, "The answer is 42");
    }

    #[test]
    fn test_job_output_expression() {
        let mut ctx = EvaluationContext::new();
        ctx.add_job_output("build".to_string(), "artifact".to_string(), "app.zip".to_string());

        let result = evaluate_expressions("Deploy ${{ ctx.jobs.build.outputs.artifact }}", &ctx).unwrap();
        assert_eq!(result, "Deploy app.zip");
    }

    #[test]
    fn test_multiple_expressions() {
        let mut ctx = EvaluationContext::new();
        ctx.inputs.insert("first".to_string(), "John".to_string());
        ctx.inputs.insert("last".to_string(), "Doe".to_string());

        let result = evaluate_expressions(
            "Name: ${{ ctx.inputs.first }} ${{ ctx.inputs.last }}",
            &ctx
        ).unwrap();
        assert_eq!(result, "Name: John Doe");
    }

    #[test]
    fn test_missing_input() {
        let ctx = EvaluationContext::new();
        let result = evaluate_expressions("Hello ${{ ctx.inputs.name }}!", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_expression_format() {
        let ctx = EvaluationContext::new();
        let result = evaluate_expressions("Invalid ${{ foo.bar }}", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_expressions() {
        let ctx = EvaluationContext::new();
        let result = evaluate_expressions("No expressions here", &ctx).unwrap();
        assert_eq!(result, "No expressions here");
    }
}
