//! Project scaffolding command implementation

use color_eyre::eyre::Result;

/// Scaffold a new project
pub async fn run(project_type: String, name: String) -> Result<()> {
    println!("Quantumn Code - Project Scaffolding");
    println!("Type: {}", project_type);
    println!("Name: {}", name);
    println!();

    match project_type.as_str() {
        "rust" | "rs" => scaffold_rust(&name),
        "python" | "py" => scaffold_python(&name),
        "node" | "js" | "ts" | "typescript" => scaffold_node(&name),
        "web" | "html" => scaffold_web(&name),
        "go" => scaffold_go(&name),
        _ => {
            println!("Unknown project type: {}", project_type);
            println!("Available types: rust, python, node, web, go");
            Ok(())
        }
    }
}

/// Scaffold a Rust project
fn scaffold_rust(name: &str) -> Result<()> {
    println!("Creating Rust project: {}", name);

    // Run cargo new
    let output = std::process::Command::new("cargo")
        .args(["new", name])
        .output()?;

    if !output.status.success() {
        println!("Failed to create project: {}", String::from_utf8_lossy(&output.stderr));
        return Ok(());
    }

    // Create additional files
    let project_path = std::path::Path::new(name);

    // Create .gitignore
    std::fs::write(
        project_path.join(".gitignore"),
        r#"/target
**/*.rs.bk
Cargo.lock
"#,
    )?;

    // Create README
    std::fs::write(
        project_path.join("README.md"),
        format!("# {}\n\nA Rust project.\n\n## Usage\n\n```\ncargo run\n```\n", name),
    )?;

    println!("✓ Created Rust project: {}", name);
    println!("  cd {} && cargo run", name);

    Ok(())
}

/// Scaffold a Python project
fn scaffold_python(name: &str) -> Result<()> {
    println!("Creating Python project: {}", name);

    // Create directory structure
    let project_path = std::path::Path::new(name);
    std::fs::create_dir_all(project_path.join("src").join(name.replace('-', "_")))?;
    std::fs::create_dir_all(project_path.join("tests"))?;

    // Create files
    std::fs::write(project_path.join(".gitignore"), "*.pyc\n__pycache__/\n.env\n.venv/\n")?;
    std::fs::write(project_path.join("requirements.txt"), "")?;
    std::fs::write(
        project_path.join("setup.py"),
        format!(r#"from setuptools import setup, find_packages

setup(
    name="{}",
    version="0.1.0",
    packages=find_packages(),
    python_requires=">=3.8",
)
"#, name),
    )?;
    std::fs::write(
        project_path.join("src").join(name.replace('-', "_")).join("__init__.py"),
        "",
    )?;
    std::fs::write(project_path.join("tests").join("__init__.py"), "")?;
    std::fs::write(
        project_path.join("README.md"),
        format!("# {}\n\nA Python project.\n", name),
    )?;

    println!("✓ Created Python project: {}", name);
    println!("  cd {} && python -m venv .venv", name);

    Ok(())
}

/// Scaffold a Node.js project
fn scaffold_node(name: &str) -> Result<()> {
    println!("Creating Node.js project: {}", name);

    // Run npm init
    let output = std::process::Command::new("npm")
        .args(["init", "-y"])
        .current_dir(std::path::Path::new(name))
        .output();

    match output {
        Ok(_) => println!("✓ Created Node.js project: {}", name),
        Err(_) => {
            // Create manually
            let project_path = std::path::Path::new(name);
            std::fs::create_dir_all(project_path)?;

            std::fs::write(
                project_path.join("package.json"),
                format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "main": "index.js",
  "scripts": {{
    "start": "node index.js",
    "test": "jest"
  }}
}}
"#, name),
            )?;
            std::fs::write(project_path.join("index.js"), "console.log('Hello, World!');\n")?;
            std::fs::write(project_path.join(".gitignore"), "node_modules/\n.env\n")?;

            println!("✓ Created Node.js project: {}", name);
        }
    }

    println!("  cd {} && npm install", name);

    Ok(())
}

/// Scaffold a web project
fn scaffold_web(name: &str) -> Result<()> {
    println!("Creating web project: {}", name);

    let project_path = std::path::Path::new(name);
    std::fs::create_dir_all(project_path.join("css"))?;
    std::fs::create_dir_all(project_path.join("js"))?;

    std::fs::write(
        project_path.join("index.html"),
        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <link rel="stylesheet" href="css/style.css">
</head>
<body>
    <h1>{}</h1>
    <script src="js/main.js"></script>
</body>
</html>
"#, name, name),
    )?;

    std::fs::write(
        project_path.join("css").join("style.css"),
        "* {{ margin: 0; padding: 0; box-sizing: border-box; }}\nbody {{ font-family: sans-serif; }}\n",
    )?;

    std::fs::write(
        project_path.join("js").join("main.js"),
        "console.log('Hello, World!');\n",
    )?;

    println!("✓ Created web project: {}", name);
    println!("  Open {}/index.html in your browser", name);

    Ok(())
}

/// Scaffold a Go project
fn scaffold_go(name: &str) -> Result<()> {
    println!("Creating Go project: {}", name);

    let project_path = std::path::Path::new(name);
    std::fs::create_dir_all(project_path)?;

    std::fs::write(
        project_path.join("go.mod"),
        format!("module {}\n\ngo 1.21\n", name),
    )?;

    std::fs::write(
        project_path.join("main.go"),
        r#"package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}
"#,
    )?;

    std::fs::write(project_path.join(".gitignore"), "*.exe\n*.exe~\n*.dll\n*.so\n*.dylib\n")?;

    println!("✓ Created Go project: {}", name);
    println!("  cd {} && go run main.go", name);

    Ok(())
}