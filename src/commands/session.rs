//! Session management commands

use color_eyre::eyre::Result;

use crate::cli::SessionCommands;

/// Run session command
pub async fn run(command: SessionCommands) -> Result<()> {
    match command {
        SessionCommands::List => list_sessions(),
        SessionCommands::Resume { id } => resume_session(id),
        SessionCommands::Save { name } => save_session(name),
        SessionCommands::Delete { id } => delete_session(id),
    }
}

/// List saved sessions
fn list_sessions() -> Result<()> {
    println!("Quantumn Code - Sessions");
    println!();

    let sessions_dir = get_sessions_dir()?;

    if !sessions_dir.exists() {
        println!("No sessions found.");
        println!("Sessions are saved automatically in interactive mode.");
        return Ok(());
    }

    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(&sessions_dir)? {
        let entry = entry?;
        if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
            if let Some(name) = entry.file_name().to_str() {
                sessions.push(name.trim_end_matches(".json").to_string());
            }
        }
    }

    if sessions.is_empty() {
        println!("No sessions found.");
    } else {
        println!("Saved sessions:");
        for session in sessions {
            println!("  - {}", session);
        }
    }

    println!("\nTo resume a session:");
    println!("  quantumn session resume <id>");

    Ok(())
}

/// Resume a session
fn resume_session(id: Option<String>) -> Result<()> {
    match id {
        Some(session_id) => {
            println!("Resuming session: {}", session_id);
            // TODO: Load session and start interactive mode
            println!("Session resumption will be implemented in Phase 3.");
        }
        None => {
            // Resume most recent
            println!("Resuming most recent session...");
            // TODO: Find and load most recent session
            println!("Session resumption will be implemented in Phase 3.");
        }
    }

    Ok(())
}

/// Save current session
fn save_session(name: Option<String>) -> Result<()> {
    let session_name = name.unwrap_or_else(|| {
        chrono::Local::now().format("session_%Y%m%d_%H%M%S").to_string()
    });

    println!("Saving session: {}", session_name);
    // TODO: Save current session state
    println!("Session saving will be implemented in Phase 3.");

    Ok(())
}

/// Delete a session
fn delete_session(id: String) -> Result<()> {
    println!("Deleting session: {}", id);

    let sessions_dir = get_sessions_dir()?;
    let session_file = sessions_dir.join(format!("{}.json", id));

    if session_file.exists() {
        std::fs::remove_file(&session_file)?;
        println!("✓ Session deleted: {}", id);
    } else {
        println!("Session not found: {}", id);
    }

    Ok(())
}

/// Get the sessions directory
fn get_sessions_dir() -> Result<std::path::PathBuf> {
    let config_dir = directories::ProjectDirs::from("com", "quantumn", "code")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from(".quantumn"));

    Ok(config_dir.join("sessions"))
}