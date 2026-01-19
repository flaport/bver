use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::finders::find_project_root;
use crate::schema::Config;

const DEFAULT_CONTEXT_LINES: usize = 3;

pub fn bump_version(config: &Config, component: &str) -> Result<(), String> {
    let current_version = config
        .current_version
        .as_ref()
        .ok_or("No current_version found in config")?;

    let new_version = compute_new_version(current_version, component)?;
    let context_lines = config.context_lines.unwrap_or(DEFAULT_CONTEXT_LINES);
    let project_root = find_project_root().ok_or("Could not find project root")?;

    println!("Bumping version: {current_version} -> {new_version}");
    println!();

    for file_config in &config.files {
        let file_path = project_root.join(&file_config.src);
        if !file_path.exists() {
            eprintln!("Warning: File not found: {}", file_path.display());
            continue;
        }

        process_file(&file_path, current_version, &new_version, context_lines)?;
    }

    Ok(())
}

fn compute_new_version(current: &str, component: &str) -> Result<String, String> {
    let parts: Vec<&str> = current.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "Invalid version format: {current}. Expected semver (major.minor.patch)"
        ));
    }

    let major: u32 = parts[0]
        .parse()
        .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
    let minor: u32 = parts[1]
        .parse()
        .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
    let patch: u32 = parts[2]
        .parse()
        .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;

    let (major, minor, patch) = match component {
        "major" => (major + 1, 0, 0),
        "minor" => (major, minor + 1, 0),
        "patch" => (major, minor, patch + 1),
        _ => return Err(format!("Invalid component: {component}. Use major, minor, or patch")),
    };

    Ok(format!("{major}.{minor}.{patch}"))
}

fn process_file(
    path: &Path,
    old_version: &str,
    new_version: &str,
    context_lines: usize,
) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let lines: Vec<&str> = content.lines().collect();

    let occurrences: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains(old_version))
        .map(|(i, _)| i)
        .collect();

    if occurrences.is_empty() {
        return Ok(());
    }

    println!("File: {}", path.display());
    println!("{}", "=".repeat(60));

    let mut accepted_lines: Vec<usize> = Vec::new();

    for &line_idx in &occurrences {
        if show_diff_and_prompt(path, &lines, line_idx, old_version, new_version, context_lines)? {
            accepted_lines.push(line_idx);
        }
    }

    if !accepted_lines.is_empty() {
        apply_changes(path, &lines, &accepted_lines, old_version, new_version)?;
    }

    println!();
    Ok(())
}

fn show_diff_and_prompt(
    _path: &Path,
    lines: &[&str],
    line_idx: usize,
    old_version: &str,
    new_version: &str,
    context_lines: usize,
) -> Result<bool, String> {
    let start = line_idx.saturating_sub(context_lines);
    let end = (line_idx + context_lines + 1).min(lines.len());

    println!();

    for i in start..end {
        let line_num = i + 1;
        let line = lines[i];

        if i == line_idx {
            // Show the old line in red
            println!(
                "\x1b[31m- {line_num:4} | {}\x1b[0m",
                line
            );
            // Show the new line in green
            let new_line = line.replace(old_version, new_version);
            println!(
                "\x1b[32m+ {line_num:4} | {}\x1b[0m",
                new_line
            );
        } else {
            println!("  {line_num:4} | {line}");
        }
    }

    println!();
    print!("Apply this change? [Y/n]: ");
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;

    let input = input.trim().to_lowercase();
    Ok(input.is_empty() || input == "y" || input == "yes")
}

fn apply_changes(
    path: &Path,
    lines: &[&str],
    accepted_lines: &[usize],
    old_version: &str,
    new_version: &str,
) -> Result<(), String> {
    let new_content: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if accepted_lines.contains(&i) {
                line.replace(old_version, new_version)
            } else {
                (*line).to_string()
            }
        })
        .collect();

    let new_content = new_content.join("\n");

    // Preserve trailing newline if original had one
    let original = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let new_content = if original.ends_with('\n') {
        new_content + "\n"
    } else {
        new_content
    };

    fs::write(path, new_content).map_err(|e| format!("Failed to write {}: {e}", path.display()))?;

    println!("Updated: {}", path.display());
    Ok(())
}
