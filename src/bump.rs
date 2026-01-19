use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::cast::cast_version;
use crate::finders::find_project_root;
use crate::schema::{Config, FileKind, OnInvalidVersion};
use crate::version::validate_version;

const DEFAULT_CONTEXT_LINES: usize = 3;

pub fn bump_version(config: &Config, target: &str) -> Result<(), String> {
    let current_version = config
        .current_version
        .as_ref()
        .ok_or("No current_version found in config")?;

    let new_version = if is_version_string(target) {
        target.to_string()
    } else {
        compute_new_version(current_version, target)?
    };
    let context_lines = config.context_lines.unwrap_or(DEFAULT_CONTEXT_LINES);
    let project_root = find_project_root().ok_or("Could not find project root")?;

    println!("Bumping version: {current_version} -> {new_version}");
    println!();

    let default_kind = config.default_kind;

    for file_config in &config.files {
        let file_path = project_root.join(&file_config.src);
        if !file_path.exists() {
            eprintln!("Warning: File not found: {}", file_path.display());
            continue;
        }

        let kind = file_config.kind.unwrap_or(default_kind);

        // Get the versions to use for this file (possibly casted)
        let old_file_version = get_file_version(current_version, kind, config.on_invalid_version, &file_config.src)?;
        let new_file_version = get_file_version(&new_version, kind, config.on_invalid_version, &file_config.src)?;

        process_file(&file_path, &old_file_version, &new_file_version, kind, context_lines)?;
    }

    Ok(())
}

fn is_version_string(s: &str) -> bool {
    !matches!(s, "major" | "minor" | "patch" | "alpha" | "beta" | "rc" | "post" | "dev" | "release")
}

fn get_file_version(
    version: &str,
    kind: FileKind,
    on_invalid: OnInvalidVersion,
    src: &Path,
) -> Result<String, String> {
    // First, check if the version is already valid for this kind
    if validate_version(version, kind).is_ok() {
        return Ok(version.to_string());
    }

    // Version is invalid for this kind
    match on_invalid {
        OnInvalidVersion::Error => {
            let err = validate_version(version, kind).unwrap_err();
            Err(format!(
                "Invalid version '{}' for file '{}' (kind: {:?}): {}",
                version,
                src.display(),
                kind,
                err
            ))
        }
        OnInvalidVersion::Cast => {
            let casted = cast_version(version, kind).map_err(|e| {
                format!(
                    "Cannot cast version '{}' for file '{}' (kind: {:?}): {}",
                    version,
                    src.display(),
                    kind,
                    e
                )
            })?;

            // Validate the casted version
            validate_version(&casted, kind).map_err(|e| {
                format!(
                    "Casted version '{}' is still invalid for file '{}' (kind: {:?}): {}",
                    casted,
                    src.display(),
                    kind,
                    e
                )
            })?;

            Ok(casted)
        }
    }
}

fn compute_new_version(current: &str, component: &str) -> Result<String, String> {
    let parsed = parse_version(current)?;

    match component {
        "major" => Ok(format!("{}.0.0", parsed.major + 1)),
        "minor" => Ok(format!("{}.{}.0", parsed.major, parsed.minor + 1)),
        "patch" => {
            // If we have a prerelease, just drop it (1.0.0a1 -> 1.0.0)
            if parsed.prerelease.is_some() || parsed.post.is_some() || parsed.dev.is_some() {
                Ok(format!("{}.{}.{}", parsed.major, parsed.minor, parsed.patch))
            } else {
                Ok(format!("{}.{}.{}", parsed.major, parsed.minor, parsed.patch + 1))
            }
        }
        "release" => {
            // Drop all prerelease/post/dev suffixes
            Ok(format!("{}.{}.{}", parsed.major, parsed.minor, parsed.patch))
        }
        "alpha" => {
            let num = match &parsed.prerelease {
                Some((kind, n)) if kind == "alpha" => n + 1,
                _ => 1,
            };
            Ok(format!("{}.{}.{}a{}", parsed.major, parsed.minor, parsed.patch, num))
        }
        "beta" => {
            let num = match &parsed.prerelease {
                Some((kind, n)) if kind == "beta" => n + 1,
                _ => 1,
            };
            Ok(format!("{}.{}.{}b{}", parsed.major, parsed.minor, parsed.patch, num))
        }
        "rc" => {
            let num = match &parsed.prerelease {
                Some((kind, n)) if kind == "rc" => n + 1,
                _ => 1,
            };
            Ok(format!("{}.{}.{}rc{}", parsed.major, parsed.minor, parsed.patch, num))
        }
        "post" => {
            let num = parsed.post.map(|n| n + 1).unwrap_or(1);
            let base = format!("{}.{}.{}", parsed.major, parsed.minor, parsed.patch);
            let pre = match &parsed.prerelease {
                Some((kind, n)) => format!("{}{}", prerelease_prefix(kind), n),
                None => String::new(),
            };
            Ok(format!("{}{}.post{}", base, pre, num))
        }
        "dev" => {
            let num = parsed.dev.map(|n| n + 1).unwrap_or(1);
            let base = format!("{}.{}.{}", parsed.major, parsed.minor, parsed.patch);
            let pre = match &parsed.prerelease {
                Some((kind, n)) => format!("{}{}", prerelease_prefix(kind), n),
                None => String::new(),
            };
            let post = parsed.post.map(|n| format!(".post{}", n)).unwrap_or_default();
            Ok(format!("{}{}{}.dev{}", base, pre, post, num))
        }
        _ => Err(format!(
            "Invalid component: {component}. Use major, minor, patch, release, alpha, beta, rc, post, or dev"
        )),
    }
}

fn prerelease_prefix(kind: &str) -> &'static str {
    match kind {
        "alpha" => "a",
        "beta" => "b",
        "rc" => "rc",
        _ => "",
    }
}

#[derive(Debug, Default)]
struct ParsedVersion {
    major: u32,
    minor: u32,
    patch: u32,
    prerelease: Option<(String, u32)>, // (kind, number) e.g., ("alpha", 1)
    post: Option<u32>,
    dev: Option<u32>,
}

fn parse_version(version: &str) -> Result<ParsedVersion, String> {
    let version = version.to_lowercase();

    // Remove epoch if present
    let version = if let Some(pos) = version.find('!') {
        &version[pos + 1..]
    } else {
        version.as_str()
    };

    // Remove local version if present
    let version = if let Some(pos) = version.find('+') {
        &version[..pos]
    } else {
        version
    };

    let mut parsed = ParsedVersion::default();

    // Find dev suffix
    let (version, dev) = if let Some(pos) = version.find(".dev") {
        let dev_part = &version[pos + 4..];
        let dev_num: u32 = dev_part.parse().unwrap_or(0);
        (&version[..pos], Some(dev_num))
    } else if let Some(pos) = version.find("dev") {
        let dev_part = &version[pos + 3..];
        let dev_num: u32 = dev_part.parse().unwrap_or(0);
        (&version[..pos], Some(dev_num))
    } else {
        (version, None)
    };
    parsed.dev = dev;

    // Find post suffix
    let (version, post) = if let Some(pos) = version.find(".post") {
        let post_part = &version[pos + 5..];
        let post_num: u32 = post_part.parse().unwrap_or(0);
        (&version[..pos], Some(post_num))
    } else if let Some(pos) = version.find("post") {
        let post_part = &version[pos + 4..];
        let post_num: u32 = post_part.parse().unwrap_or(0);
        (&version[..pos], Some(post_num))
    } else {
        (version, None)
    };
    parsed.post = post;

    // Find prerelease suffix (alpha, beta, rc, a, b, c)
    let prerelease_markers = [
        ("alpha", "alpha"),
        ("beta", "beta"),
        ("preview", "rc"),
        ("rc", "rc"),
        ("a", "alpha"),
        ("b", "beta"),
        ("c", "rc"),
    ];

    let mut release = version;
    for (marker, kind) in prerelease_markers {
        if let Some(pos) = version.find(marker) {
            let before = &version[..pos];
            // Make sure it's at a valid position (after a digit or dot)
            if before.is_empty() || (!before.ends_with('.') && !before.chars().last().unwrap().is_ascii_digit()) {
                continue;
            }
            let after = &version[pos + marker.len()..];
            let num: u32 = after
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0);
            parsed.prerelease = Some((kind.to_string(), num));
            release = before;
            break;
        }
    }

    // Also handle JS-style prerelease (1.0.0-alpha.1)
    let release = if let Some(pos) = release.find('-') {
        let pre_part = &release[pos + 1..];
        if pre_part.starts_with("alpha") {
            let num_part = pre_part.strip_prefix("alpha").unwrap_or("").trim_start_matches('.');
            let num: u32 = num_part.parse().unwrap_or(0);
            parsed.prerelease = Some(("alpha".to_string(), num));
        } else if pre_part.starts_with("beta") {
            let num_part = pre_part.strip_prefix("beta").unwrap_or("").trim_start_matches('.');
            let num: u32 = num_part.parse().unwrap_or(0);
            parsed.prerelease = Some(("beta".to_string(), num));
        } else if pre_part.starts_with("rc") {
            let num_part = pre_part.strip_prefix("rc").unwrap_or("").trim_start_matches('.');
            let num: u32 = num_part.parse().unwrap_or(0);
            parsed.prerelease = Some(("rc".to_string(), num));
        }
        &release[..pos]
    } else {
        release
    };

    // Parse major.minor.patch
    let parts: Vec<&str> = release.split('.').collect();
    if parts.is_empty() {
        return Err(format!("Invalid version format: {version}"));
    }

    parsed.major = parts[0]
        .parse()
        .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
    parsed.minor = parts.get(1).unwrap_or(&"0")
        .parse()
        .map_err(|_| format!("Invalid minor version: {}", parts.get(1).unwrap_or(&"0")))?;
    parsed.patch = parts.get(2).unwrap_or(&"0")
        .parse()
        .map_err(|_| format!("Invalid patch version: {}", parts.get(2).unwrap_or(&"0")))?;

    Ok(parsed)
}

fn process_file(
    path: &Path,
    old_version: &str,
    new_version: &str,
    _kind: FileKind,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_major() {
        assert_eq!(compute_new_version("1.2.3", "major").unwrap(), "2.0.0");
        assert_eq!(compute_new_version("0.1.0", "major").unwrap(), "1.0.0");
        assert_eq!(compute_new_version("1.2.3a1", "major").unwrap(), "2.0.0");
    }

    #[test]
    fn test_bump_minor() {
        assert_eq!(compute_new_version("1.2.3", "minor").unwrap(), "1.3.0");
        assert_eq!(compute_new_version("0.1.0", "minor").unwrap(), "0.2.0");
        assert_eq!(compute_new_version("1.2.3a1", "minor").unwrap(), "1.3.0");
    }

    #[test]
    fn test_bump_patch() {
        assert_eq!(compute_new_version("1.2.3", "patch").unwrap(), "1.2.4");
        assert_eq!(compute_new_version("0.1.0", "patch").unwrap(), "0.1.1");
        // With prerelease, patch just drops the prerelease
        assert_eq!(compute_new_version("1.2.3a1", "patch").unwrap(), "1.2.3");
        assert_eq!(compute_new_version("1.2.3.post1", "patch").unwrap(), "1.2.3");
    }

    #[test]
    fn test_bump_release() {
        assert_eq!(compute_new_version("1.2.3a1", "release").unwrap(), "1.2.3");
        assert_eq!(compute_new_version("1.2.3b2", "release").unwrap(), "1.2.3");
        assert_eq!(compute_new_version("1.2.3rc1", "release").unwrap(), "1.2.3");
        assert_eq!(compute_new_version("1.2.3.post1", "release").unwrap(), "1.2.3");
        assert_eq!(compute_new_version("1.2.3.dev1", "release").unwrap(), "1.2.3");
        assert_eq!(compute_new_version("1.2.3", "release").unwrap(), "1.2.3");
    }

    #[test]
    fn test_bump_alpha() {
        assert_eq!(compute_new_version("1.2.3", "alpha").unwrap(), "1.2.3a1");
        assert_eq!(compute_new_version("1.2.3a1", "alpha").unwrap(), "1.2.3a2");
        assert_eq!(compute_new_version("1.2.3a5", "alpha").unwrap(), "1.2.3a6");
        // Switching from beta/rc to alpha resets to 1
        assert_eq!(compute_new_version("1.2.3b1", "alpha").unwrap(), "1.2.3a1");
    }

    #[test]
    fn test_bump_beta() {
        assert_eq!(compute_new_version("1.2.3", "beta").unwrap(), "1.2.3b1");
        assert_eq!(compute_new_version("1.2.3b1", "beta").unwrap(), "1.2.3b2");
        assert_eq!(compute_new_version("1.2.3a1", "beta").unwrap(), "1.2.3b1");
    }

    #[test]
    fn test_bump_rc() {
        assert_eq!(compute_new_version("1.2.3", "rc").unwrap(), "1.2.3rc1");
        assert_eq!(compute_new_version("1.2.3rc1", "rc").unwrap(), "1.2.3rc2");
        assert_eq!(compute_new_version("1.2.3b1", "rc").unwrap(), "1.2.3rc1");
    }

    #[test]
    fn test_bump_post() {
        assert_eq!(compute_new_version("1.2.3", "post").unwrap(), "1.2.3.post1");
        assert_eq!(compute_new_version("1.2.3.post1", "post").unwrap(), "1.2.3.post2");
        assert_eq!(compute_new_version("1.2.3a1", "post").unwrap(), "1.2.3a1.post1");
    }

    #[test]
    fn test_bump_dev() {
        assert_eq!(compute_new_version("1.2.3", "dev").unwrap(), "1.2.3.dev1");
        assert_eq!(compute_new_version("1.2.3.dev1", "dev").unwrap(), "1.2.3.dev2");
        assert_eq!(compute_new_version("1.2.3a1", "dev").unwrap(), "1.2.3a1.dev1");
        assert_eq!(compute_new_version("1.2.3.post1", "dev").unwrap(), "1.2.3.post1.dev1");
    }

    #[test]
    fn test_bump_js_style_prerelease() {
        // JS style: 1.0.0-alpha.1
        assert_eq!(compute_new_version("1.2.3-alpha.1", "alpha").unwrap(), "1.2.3a2");
        assert_eq!(compute_new_version("1.2.3-beta.1", "beta").unwrap(), "1.2.3b2");
        assert_eq!(compute_new_version("1.2.3-rc.1", "rc").unwrap(), "1.2.3rc2");
    }

    #[test]
    fn test_parse_version() {
        let p = parse_version("1.2.3").unwrap();
        assert_eq!((p.major, p.minor, p.patch), (1, 2, 3));
        assert!(p.prerelease.is_none());

        let p = parse_version("1.2.3a1").unwrap();
        assert_eq!((p.major, p.minor, p.patch), (1, 2, 3));
        assert_eq!(p.prerelease, Some(("alpha".to_string(), 1)));

        let p = parse_version("1.2.3.post1").unwrap();
        assert_eq!(p.post, Some(1));

        let p = parse_version("1.2.3.dev1").unwrap();
        assert_eq!(p.dev, Some(1));
    }
}
