//! TUI for real-time `ferri flow run` execution.

use ferri_core::flow::Pipeline;
use ratatui::{prelude::*, widgets::*};
use std::io;

pub fn run(pipeline: &Pipeline) -> io::Result<()> {
    // For now, this is a placeholder that just prints the steps.
    // The next tickets will build this out into a full TUI.
    println!("--- Executing Flow: {} ---", pipeline.name);
    for (i, step) in pipeline.steps.iter().enumerate() {
        println!("[Step {}] {}: Pending", i + 1, step.name);
    }
    println!("\n(TUI implementation pending)");
    Ok(())
}

