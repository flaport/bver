use std::path::PathBuf;

pub fn find_repo_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn find_pyproject_toml() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        let pyproject = current.join("pyproject.toml");
        if pyproject.exists() {
            return Some(pyproject);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn find_package_json() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        let package_json = current.join("package.json");
        if package_json.exists() {
            return Some(package_json);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn find_cargo_toml() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Some(cargo_toml);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn find_bver_toml() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        let bver_toml = current.join("bver.toml");
        if bver_toml.exists() {
            return Some(bver_toml);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn find_project_root() -> Option<PathBuf> {
    find_repo_root()
        .or_else(|| find_bver_toml().and_then(|p| p.parent().map(PathBuf::from)))
        .or_else(|| find_pyproject_toml().and_then(|p| p.parent().map(PathBuf::from)))
        .or_else(|| find_package_json().and_then(|p| p.parent().map(PathBuf::from)))
        .or_else(|| find_cargo_toml().and_then(|p| p.parent().map(PathBuf::from)))
}
