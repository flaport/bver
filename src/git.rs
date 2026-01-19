use std::process::Command;

use crate::finders::find_repo_root;
use crate::schema::{GitAction, RunPreCommit};

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

/// Run a git command and return the result
fn git(args: &[&str]) -> Result<(), String> {
    println!("Running: git {}", args.join(" "));

    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git {} failed: {}", args[0], stderr.trim()));
    }

    Ok(())
}

/// Run git operations based on config setting
pub fn run_git_actions(
    action: GitAction,
    old_version: &str,
    new_version: &str,
) -> Result<(), String> {
    match action {
        GitAction::Disabled => Ok(()),
        GitAction::Commit => {
            git_add_all()?;
            git_commit(old_version, new_version)?;
            Ok(())
        }
        GitAction::CommitAndTag => {
            git_add_all()?;
            git_commit(old_version, new_version)?;
            git_tag(new_version)?;
            Ok(())
        }
        GitAction::CommitTagAndPush => {
            git_add_all()?;
            git_commit(old_version, new_version)?;
            git_tag(new_version)?;
            git_push()?;
            git_push_tag(new_version)?;
            Ok(())
        }
    }
}

fn git_add_all() -> Result<(), String> {
    git(&["add", "--all"])
}

fn git_commit(old_version: &str, new_version: &str) -> Result<(), String> {
    let msg = format!("Bump version from {} to {}", old_version, new_version);
    git(&["commit", "-m", &msg])
}

fn git_tag(version: &str) -> Result<(), String> {
    let msg = format!("Release {}", version);
    git(&["tag", "-a", version, "-m", &msg])
}

fn git_push() -> Result<(), String> {
    git(&["push"])
}

fn git_push_tag(version: &str) -> Result<(), String> {
    git(&["push", "origin", version])
}
