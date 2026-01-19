use crate::schema::FileKind;

/// Validate a version string according to the file kind
pub fn validate_version(version: &str, kind: FileKind) -> Result<(), String> {
    match kind {
        FileKind::Any => Ok(()),
        FileKind::Simple => validate_simple(version),
        FileKind::Python => validate_python(version),
        FileKind::Javascript => validate_javascript(version),
    }
}

/// Validate a simple semver version (N.N.N)
fn validate_simple(version: &str) -> Result<(), String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "Invalid simple version: {version}. Expected format: major.minor.patch"
        ));
    }
    for (i, part) in parts.iter().enumerate() {
        let name = ["major", "minor", "patch"][i];
        if part.parse::<u32>().is_err() {
            return Err(format!("Invalid {name} version component: {part}"));
        }
    }
    Ok(())
}

/// Validate a Python version string (PEP 440)
/// https://peps.python.org/pep-0440/
///
/// Valid forms:
/// - N[.N]+                           (e.g., 1.0, 1.0.0, 1.2.3.4)
/// - N[.N]+{a|b|rc}N                  (e.g., 1.0a1, 1.0b2, 1.0rc1)
/// - N[.N]+.postN                     (e.g., 1.0.post1)
/// - N[.N]+.devN                      (e.g., 1.0.dev1)
/// - N[.N]+{a|b|rc}N.postN            (e.g., 1.0a1.post1)
/// - N[.N]+{a|b|rc}N.devN             (e.g., 1.0a1.dev1)
/// - N[.N]+.postN.devN                (e.g., 1.0.post1.dev1)
/// - N[.N]+{a|b|rc}N.postN.devN       (e.g., 1.0a1.post1.dev1)
/// - Any of the above with +local     (e.g., 1.0+local.version)
/// - Any of the above with N! prefix  (e.g., 1!1.0)
fn validate_python(version: &str) -> Result<(), String> {
    if version.is_empty() {
        return Err("Version string cannot be empty".to_string());
    }

    let version = version.to_lowercase();

    // Handle epoch (e.g., "1!1.0")
    let version = if let Some(pos) = version.find('!') {
        let epoch = &version[..pos];
        if !epoch.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("Invalid epoch: {epoch}"));
        }
        &version[pos + 1..]
    } else {
        version.as_str()
    };

    // Handle local version (e.g., "1.0+local")
    let version = if let Some(pos) = version.find('+') {
        let local = &version[pos + 1..];
        if !is_valid_local(local) {
            return Err(format!("Invalid local version: {local}"));
        }
        &version[..pos]
    } else {
        version
    };

    // Parse the main version parts
    parse_main_version(version)
}

fn is_valid_local(local: &str) -> bool {
    if local.is_empty() {
        return false;
    }
    // Local version: alphanumerics and dots, segments separated by dots
    local
        .split('.')
        .all(|segment| !segment.is_empty() && segment.chars().all(|c| c.is_ascii_alphanumeric()))
}

fn parse_main_version(version: &str) -> Result<(), String> {
    if version.is_empty() {
        return Err("Version string cannot be empty".to_string());
    }

    // Try to find pre-release marker (a, b, rc, alpha, beta, preview, c)
    let (release_part, remainder) = split_at_prerelease(version);

    // Validate release part (N.N.N...)
    if !is_valid_release(release_part) {
        return Err(format!("Invalid release version: {release_part}"));
    }

    if remainder.is_empty() {
        return Ok(());
    }

    // Parse pre-release, post-release, and dev markers
    parse_suffixes(remainder)
}

fn split_at_prerelease(version: &str) -> (&str, &str) {
    // Find first occurrence of pre-release markers
    let markers = ["alpha", "beta", "preview", "rc", "a", "b", "c"];

    let mut earliest_pos = None;

    for marker in markers {
        if let Some(pos) = version.find(marker) {
            // Make sure it's not part of a segment (e.g., "1.0abc" should not match)
            let before = &version[..pos];
            if before.is_empty() || before.ends_with('.') || before.chars().last().unwrap().is_ascii_digit() {
                match earliest_pos {
                    None => earliest_pos = Some(pos),
                    Some(current) if pos < current => earliest_pos = Some(pos),
                    _ => {}
                }
            }
        }
    }

    // Also check for .post and .dev at the start of suffix
    if let Some(pos) = version.find(".post") {
        match earliest_pos {
            None => earliest_pos = Some(pos),
            Some(current) if pos < current => earliest_pos = Some(pos),
            _ => {}
        }
    }
    if let Some(pos) = version.find(".dev") {
        match earliest_pos {
            None => earliest_pos = Some(pos),
            Some(current) if pos < current => earliest_pos = Some(pos),
            _ => {}
        }
    }

    match earliest_pos {
        Some(pos) => (&version[..pos], &version[pos..]),
        None => (version, ""),
    }
}

fn is_valid_release(release: &str) -> bool {
    if release.is_empty() {
        return false;
    }

    // Must be N or N.N or N.N.N etc.
    release.split('.').all(|part| {
        !part.is_empty() && part.chars().all(|c| c.is_ascii_digit())
    })
}

/// Validate a JavaScript/npm version string
/// https://semver.org/
///
/// Format: major.minor.patch[-prerelease][+build]
/// - Prerelease: -alpha.1, -beta.2, -rc.1, etc.
/// - Build metadata: +build.123 (ignored for precedence)
///
/// Note: post and dev releases are NOT supported in npm
fn validate_javascript(version: &str) -> Result<(), String> {
    if version.is_empty() {
        return Err("Version string cannot be empty".to_string());
    }

    // Split off build metadata (e.g., "1.0.0+build")
    let version = if let Some(pos) = version.find('+') {
        let build = &version[pos + 1..];
        if build.is_empty() || !is_valid_semver_identifier(build) {
            return Err(format!("Invalid build metadata: {build}"));
        }
        &version[..pos]
    } else {
        version
    };

    // Split off prerelease (e.g., "1.0.0-alpha.1")
    let (release, prerelease) = if let Some(pos) = version.find('-') {
        (&version[..pos], Some(&version[pos + 1..]))
    } else {
        (version, None)
    };

    // Validate release part (must be exactly major.minor.patch)
    let parts: Vec<&str> = release.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "Invalid semver: {release}. Expected format: major.minor.patch"
        ));
    }
    for (i, part) in parts.iter().enumerate() {
        let name = ["major", "minor", "patch"][i];
        if part.parse::<u32>().is_err() {
            return Err(format!("Invalid {name} version: {part}"));
        }
    }

    // Validate prerelease if present
    if let Some(pre) = prerelease {
        if pre.is_empty() || !is_valid_semver_identifier(pre) {
            return Err(format!("Invalid prerelease: {pre}"));
        }
    }

    Ok(())
}

fn is_valid_semver_identifier(id: &str) -> bool {
    // Identifiers are dot-separated, each part is alphanumeric or hyphen
    id.split('.').all(|part| {
        !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    })
}

fn parse_suffixes(suffix: &str) -> Result<(), String> {
    if suffix.is_empty() {
        return Ok(());
    }

    let suffix = suffix.to_lowercase();
    let mut remaining = suffix.as_str();

    // Parse pre-release (a, b, rc, alpha, beta, preview, c)
    let pre_markers = [
        ("alpha", "a"),
        ("beta", "b"),
        ("preview", "rc"),
        ("rc", "rc"),
        ("a", "a"),
        ("b", "b"),
        ("c", "rc"),
    ];

    for (marker, _normalized) in pre_markers {
        if remaining.starts_with(marker) {
            remaining = &remaining[marker.len()..];
            // Consume optional number
            let num_end = remaining
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .count();
            remaining = &remaining[num_end..];
            break;
        }
    }

    // Parse .post
    if remaining.starts_with(".post") || remaining.starts_with("post") {
        remaining = remaining.trim_start_matches('.').trim_start_matches("post");
        let num_end = remaining
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .count();
        remaining = &remaining[num_end..];
    }

    // Parse .dev
    if remaining.starts_with(".dev") || remaining.starts_with("dev") {
        remaining = remaining.trim_start_matches('.').trim_start_matches("dev");
        let num_end = remaining
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .count();
        remaining = &remaining[num_end..];
    }

    if remaining.is_empty() {
        Ok(())
    } else {
        Err(format!("Invalid version suffix: {remaining}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_python_versions() {
        assert!(validate_python("1").is_ok());
        assert!(validate_python("1.0").is_ok());
        assert!(validate_python("1.0.0").is_ok());
        assert!(validate_python("1.2.3").is_ok());
        assert!(validate_python("1.2.3.4").is_ok());
        assert!(validate_python("0.0.1").is_ok());
        assert!(validate_python("10.20.30").is_ok());
    }

    #[test]
    fn test_valid_python_prerelease_versions() {
        assert!(validate_python("1.0a1").is_ok());
        assert!(validate_python("1.0b2").is_ok());
        assert!(validate_python("1.0rc1").is_ok());
        assert!(validate_python("1.0alpha1").is_ok());
        assert!(validate_python("1.0beta2").is_ok());
        assert!(validate_python("1.0.0a1").is_ok());
        assert!(validate_python("1.0c1").is_ok());
        assert!(validate_python("1.0preview1").is_ok());
    }

    #[test]
    fn test_valid_python_post_versions() {
        assert!(validate_python("1.0.post1").is_ok());
        assert!(validate_python("1.0.0.post1").is_ok());
        assert!(validate_python("1.0a1.post1").is_ok());
    }

    #[test]
    fn test_valid_python_dev_versions() {
        assert!(validate_python("1.0.dev1").is_ok());
        assert!(validate_python("1.0.0.dev1").is_ok());
        assert!(validate_python("1.0a1.dev1").is_ok());
        assert!(validate_python("1.0.post1.dev1").is_ok());
    }

    #[test]
    fn test_valid_python_epoch_versions() {
        assert!(validate_python("1!1.0").is_ok());
        assert!(validate_python("2!1.0.0").is_ok());
    }

    #[test]
    fn test_valid_python_local_versions() {
        assert!(validate_python("1.0+local").is_ok());
        assert!(validate_python("1.0+local.version").is_ok());
        assert!(validate_python("1.0+abc123").is_ok());
        assert!(validate_python("1.0a1+local").is_ok());
    }

    #[test]
    fn test_invalid_python_versions() {
        assert!(validate_python("").is_err());
        assert!(validate_python("a.b.c").is_err());
        assert!(validate_python("1.0+").is_err());
        assert!(validate_python("1.0.").is_err());
        assert!(validate_python(".1.0").is_err());
        assert!(validate_python("1..0").is_err());
    }

    #[test]
    fn test_valid_javascript_versions() {
        assert!(validate_javascript("1.0.0").is_ok());
        assert!(validate_javascript("1.2.3").is_ok());
        assert!(validate_javascript("0.0.1").is_ok());
        assert!(validate_javascript("10.20.30").is_ok());
    }

    #[test]
    fn test_valid_javascript_prerelease_versions() {
        assert!(validate_javascript("1.0.0-alpha.1").is_ok());
        assert!(validate_javascript("1.0.0-beta.2").is_ok());
        assert!(validate_javascript("1.0.0-rc.1").is_ok());
        assert!(validate_javascript("1.0.0-0").is_ok());
        assert!(validate_javascript("1.0.0-alpha").is_ok());
    }

    #[test]
    fn test_valid_javascript_build_versions() {
        assert!(validate_javascript("1.0.0+build").is_ok());
        assert!(validate_javascript("1.0.0+build.123").is_ok());
        assert!(validate_javascript("1.0.0-alpha.1+build").is_ok());
    }

    #[test]
    fn test_invalid_javascript_versions() {
        assert!(validate_javascript("").is_err());
        assert!(validate_javascript("1.0").is_err()); // Must have 3 parts
        assert!(validate_javascript("1").is_err());
        assert!(validate_javascript("1.0.0-").is_err());
        assert!(validate_javascript("1.0.0+").is_err());
    }
}
