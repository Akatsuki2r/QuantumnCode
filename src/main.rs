//! Quantumn Code - An advanced AI-powered coding assistant CLI
//!
//! Features:
//! - Multi-provider AI support (Claude, OpenAI, Ollama)
//! - Interactive TUI with beautiful themes
//! - Git integration with AI-generated commits
//! - Code generation and editing
//! - Test runner with AI analysis
//! - Project scaffolding

use clap::Parser;
use color_eyre::eyre::Result;

mod cli;
mod app;
mod config;
mod providers;
mod commands;
mod tui;
mod tools;
mod utils;

use cli::{Cli, Commands};

/// Main entry point
fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install()?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    // Run the CLI
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        run(cli).await
    })
}

/// Main async runner
async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        // Start interactive session
        None => {
            tui::run_interactive(cli.model, cli.theme).await
        }

        // One-shot query mode
        Some(Commands::Chat { prompt, model }) => {
            commands::chat::run(prompt, model.or(cli.model)).await
        }

        Some(Commands::Edit { file, prompt, model }) => {
            commands::edit::run(file, prompt, model.or(cli.model)).await
        }

        Some(Commands::Commit { message, model }) => {
            commands::git::commit(message, model.or(cli.model)).await
        }

        Some(Commands::Review { files, model }) => {
            commands::review::run(files, model.or(cli.model)).await
        }

        Some(Commands::Test { path, model }) => {
            commands::test::run(path, model.or(cli.model)).await
        }

        Some(Commands::Scaffold { project_type, name }) => {
            commands::scaffold::run(project_type, name).await
        }

        Some(Commands::Session { command }) => {
            commands::session::run(command).await
        }

        Some(Commands::Config { command }) => {
            commands::config::run(command).await
        }

        Some(Commands::Theme { command }) => {
            commands::theme::run(command).await
        }

        Some(Commands::Model { provider, list }) => {
            commands::model::run(provider, list).await
        }

        Some(Commands::Status) => {
            commands::status::run().await
        }

        Some(Commands::Version) => {
            println!("Quantumn Code v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}