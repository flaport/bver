use std::process::Command;

use crate::finders::find_repo_root;
use crate::schema::RunPreCommit;

/// Check if pre-commit is available (installed and hook exists in .git)
fn pre_commit_available() -> bool {
    // Check if pre-commit command is available
    let cmd_available = Command::new("pre-commit")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !cmd_available {
        return false;
    }

    // Check if .git/hooks/pre-commit exists
    if let Some(repo_root) = find_repo_root() {
        repo_root.join(".git/hooks/pre-commit").exists()
    } else {
        false
    }
}

/// Run pre-commit hooks based on config setting
pub fn maybe_run_pre_commit(setting: RunPreCommit) -> Result<(), String> {
    match setting {
        RunPreCommit::Disabled => Ok(()),
        RunPreCommit::Enabled => run_pre_commit(true),
        RunPreCommit::WhenPresent => run_pre_commit(false),
    }
}

fn run_pre_commit(required: bool) -> Result<(), String> {
    if !pre_commit_available() {
        if required {
            return Err("pre-commit is not installed but run-pre-commit is enabled".to_string());
        }
        return Ok(());
    }

    println!("Running pre-commit hooks...");

    let status = Command::new("pre-commit")
        .args(["run", "--all-files"])
        .status()
        .map_err(|e| format!("Failed to run pre-commit: {e}"))?;

    if status.success() {
        println!("Pre-commit hooks passed.");
    } else {
        // Pre-commit failed, but that's often expected (it may have fixed files)
        println!("Pre-commit hooks made changes or had warnings.");
    }

    Ok(())
}
