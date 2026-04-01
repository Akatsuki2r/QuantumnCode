//! File writing tool

use std::path::Path;
use color_eyre::eyre::Result;

/// Write content to file
pub fn write_file(path: &Path, content: &str) -> Result<()> {
    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create directory {:?}: {}", parent, e))?;
    }

    std::fs::write(path, content)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to write file {:?}: {}", path, e))
}

/// Append to file
pub fn append_file(path: &Path, content: &str) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to open file {:?}: {}", path, e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| color_eyre::eyre::eyre!("Failed to append to file {:?}: {}", path, e))
}

/// Edit file at line
pub fn edit_line(path: &Path, line_num: usize, new_content: &str) -> Result<()> {
    let content = super::read_file::read_file(path)?;
    let mut lines: Vec<&str> = content.lines().collect();

    if line_num == 0 || line_num > lines.len() {
        return Err(color_eyre::eyre::eyre!("Invalid line number: {}", line_num));
    }

    lines[line_num - 1] = new_content;
    write_file(path, &lines.join("\n"))
}