use crate::schema::FileKind;

/// Cast a version string to the target kind, potentially losing information.
/// Returns the casted version string or an error if casting is not possible.
pub fn cast_version(version: &str, target_kind: FileKind) -> Result<String, String> {
    match target_kind {
        FileKind::Any => Ok(version.to_string()),
        FileKind::Simple => cast_to_simple(version),
        FileKind::Python => cast_to_python(version),
        FileKind::Semver => cast_to_semver(version),
    }
}

/// Cast any version to simple semver (major.minor.patch).
/// Strips pre-release, post-release, dev, local, and epoch information.
fn cast_to_simple(version: &str) -> Result<String, String> {
    let version = version.to_lowercase();

    // Remove epoch (e.g., "1!1.0" -> "1.0")
    let version = if let Some(pos) = version.find('!') {
        &version[pos + 1..]
    } else {
        version.as_str()
    };

    // Remove local version (e.g., "1.0+local" -> "1.0")
    let version = if let Some(pos) = version.find('+') {
        &version[..pos]
    } else {
        version
    };

    // Find where the release version ends (before any pre/post/dev markers)
    let release_end = find_release_end(version);
    let release = &version[..release_end];

    // Parse the release parts
    let parts: Vec<&str> = release.split('.').collect();

    if parts.is_empty() {
        return Err(format!("Cannot cast '{version}' to simple version: no version parts found"));
    }

    // Validate all parts are numeric
    for part in &parts {
        if part.parse::<u32>().is_err() {
            return Err(format!("Cannot cast '{version}' to simple version: invalid part '{part}'"));
        }
    }

    // Pad or truncate to exactly 3 parts
    let major = parts.first().unwrap_or(&"0");
    let minor = parts.get(1).unwrap_or(&"0");
    let patch = parts.get(2).unwrap_or(&"0");

    Ok(format!("{major}.{minor}.{patch}"))
}

/// Cast any version to PEP 440 format.
/// Most versions are already valid or can be normalized.
fn cast_to_python(version: &str) -> Result<String, String> {
    // Simple semver is valid PEP 440
    let parts: Vec<&str> = version.split('.').collect();
    if parts.iter().all(|p| p.parse::<u32>().is_ok()) {
        return Ok(version.to_string());
    }

    // Already a valid Python version (assume it's fine)
    // The validator will catch any issues
    Ok(version.to_string())
}

/// Cast any version to semver format (used by npm, Cargo, etc.).
/// Converts Python-style prereleases to semver-style (e.g., 1.2.3a1 -> 1.2.3-alpha.1)
/// Strips post and dev releases as they're not supported in semver.
fn cast_to_semver(version: &str) -> Result<String, String> {
    let version = version.to_lowercase();

    // Remove epoch (e.g., "1!1.0" -> "1.0")
    let version = if let Some(pos) = version.find('!') {
        &version[pos + 1..]
    } else {
        version.as_str()
    };

    // Remove local version (e.g., "1.0+local" -> "1.0")
    let version = if let Some(pos) = version.find('+') {
        &version[..pos]
    } else {
        version
    };

    // Find where the release version ends
    let release_end = find_release_end(version);
    let release = &version[..release_end];
    let suffix = &version[release_end..];

    // Parse the release parts and ensure we have exactly 3
    let parts: Vec<&str> = release.split('.').collect();
    if parts.is_empty() {
        return Err(format!("Cannot cast '{version}' to semver: no version parts found"));
    }

    for part in &parts {
        if part.parse::<u32>().is_err() {
            return Err(format!("Cannot cast '{version}' to semver: invalid part '{part}'"));
        }
    }

    let major = parts.first().unwrap_or(&"0");
    let minor = parts.get(1).unwrap_or(&"0");
    let patch = parts.get(2).unwrap_or(&"0");
    let base = format!("{major}.{minor}.{patch}");

    // Convert Python prerelease to JS format
    if suffix.is_empty() {
        return Ok(base);
    }

    // Strip .post and .dev as they're not supported
    let suffix = suffix
        .split(".post")
        .next()
        .unwrap_or(suffix)
        .split(".dev")
        .next()
        .unwrap_or(suffix);

    if suffix.is_empty() {
        return Ok(base);
    }

    // Convert a1 -> -alpha.1, b1 -> -beta.1, rc1 -> -rc.1
    let js_prerelease = if let Some(rest) = suffix.strip_prefix("alpha") {
        format!("-alpha.{}", rest.trim_start_matches(|c: char| !c.is_ascii_digit()))
    } else if let Some(rest) = suffix.strip_prefix('a') {
        format!("-alpha.{}", rest.trim_start_matches(|c: char| !c.is_ascii_digit()))
    } else if let Some(rest) = suffix.strip_prefix("beta") {
        format!("-beta.{}", rest.trim_start_matches(|c: char| !c.is_ascii_digit()))
    } else if let Some(rest) = suffix.strip_prefix('b') {
        format!("-beta.{}", rest.trim_start_matches(|c: char| !c.is_ascii_digit()))
    } else if let Some(rest) = suffix.strip_prefix("rc") {
        format!("-rc.{}", rest.trim_start_matches(|c: char| !c.is_ascii_digit()))
    } else if let Some(rest) = suffix.strip_prefix('c') {
        format!("-rc.{}", rest.trim_start_matches(|c: char| !c.is_ascii_digit()))
    } else if let Some(rest) = suffix.strip_prefix("preview") {
        format!("-rc.{}", rest.trim_start_matches(|c: char| !c.is_ascii_digit()))
    } else {
        // Unknown suffix, strip it
        return Ok(base);
    };

    Ok(format!("{base}{js_prerelease}"))
}

/// Find the end position of the release version (before pre/post/dev markers).
fn find_release_end(version: &str) -> usize {
    let markers = ["a", "b", "c", "alpha", "beta", "preview", "rc", ".post", ".dev", "-"];

    let mut earliest = version.len();

    for marker in markers {
        if let Some(pos) = version.find(marker) {
            // Make sure we're not matching in the middle of a number
            let before = &version[..pos];
            if before.is_empty() || before.ends_with('.') || before.chars().last().unwrap().is_ascii_digit() {
                earliest = earliest.min(pos);
            }
        }
    }

    earliest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cast_to_simple() {
        // Already simple
        assert_eq!(cast_to_simple("1.2.3").unwrap(), "1.2.3");

        // Pad missing parts
        assert_eq!(cast_to_simple("1").unwrap(), "1.0.0");
        assert_eq!(cast_to_simple("1.2").unwrap(), "1.2.0");

        // Strip pre-release
        assert_eq!(cast_to_simple("1.2.3a1").unwrap(), "1.2.3");
        assert_eq!(cast_to_simple("1.2.3b2").unwrap(), "1.2.3");
        assert_eq!(cast_to_simple("1.2.3rc1").unwrap(), "1.2.3");
        assert_eq!(cast_to_simple("1.2.3alpha1").unwrap(), "1.2.3");
        assert_eq!(cast_to_simple("1.2.3beta2").unwrap(), "1.2.3");

        // Strip post-release
        assert_eq!(cast_to_simple("1.2.3.post1").unwrap(), "1.2.3");

        // Strip dev
        assert_eq!(cast_to_simple("1.2.3.dev1").unwrap(), "1.2.3");

        // Strip local
        assert_eq!(cast_to_simple("1.2.3+local").unwrap(), "1.2.3");

        // Strip epoch
        assert_eq!(cast_to_simple("1!1.2.3").unwrap(), "1.2.3");

        // Combined
        assert_eq!(cast_to_simple("1!1.2.3a1.post1.dev1+local").unwrap(), "1.2.3");
    }

    #[test]
    fn test_cast_to_python() {
        // Simple versions pass through
        assert_eq!(cast_to_python("1.2.3").unwrap(), "1.2.3");

        // Python versions pass through
        assert_eq!(cast_to_python("1.2.3a1").unwrap(), "1.2.3a1");
    }
}
