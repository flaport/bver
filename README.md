# bver 0.0.1
> A bump-version tool for multi-language projects

![bver-logo](img/bver-small.png)

---

## Features

- **Multi-format version support**: Handles Python (PEP 440), Semver (npm/Cargo), and simple `major.minor.patch` versions
- **Automatic version casting**: Convert between version formats when needed (e.g., `1.2.3a1` to `1.2.3-alpha.1`)
- **Interactive TUI**: Review and selectively apply version changes with a terminal UI
- **Git integration**: Automatic commits, tags, and pushes
- **Pre-commit hook support**: Run pre-commit hooks before committing version bumps
- **Flexible configuration**: Configure via `bver.toml`, `pyproject.toml`, `package.json`, or `Cargo.toml`

## Installation

```bash
cargo install bver
```

## Usage

```bash
# Show current version
bver current

# Show full configuration
bver config

# Bump version
bver bump patch          # 1.2.3 -> 1.2.4
bver bump minor          # 1.2.3 -> 1.3.0
bver bump major          # 1.2.3 -> 2.0.0

# Pre-release versions
bver bump alpha          # 1.2.3 -> 1.2.3a1
bver bump beta           # 1.2.3 -> 1.2.3b1
bver bump rc             # 1.2.3 -> 1.2.3rc1

# Post-release and dev versions
bver bump post           # 1.2.3 -> 1.2.3.post1
bver bump dev            # 1.2.3 -> 1.2.3.dev1

# Release (strip pre-release suffix)
bver bump release        # 1.2.3a1 -> 1.2.3

# Set explicit version
bver bump 2.0.0

# Force git operations (overwrite tags, force push)
bver bump patch --force
```

## Configuration

### Standalone (`bver.toml`)

```toml
current-version = "1.2.3"

# Optional settings
context-lines = 3              # Lines of context in diff preview
default-kind = "any"           # any | simple | python | semver
on-invalid-version = "error"   # error | cast
run-pre-commit = "when-present" # enabled | disabled | when-present
git-action = "commit-and-tag"  # disabled | commit | commit-and-tag | commit-tag-and-push

[[file]]
src = "src/version.py"
kind = "python" # pep440

[[file]]
src = "package.json"
kind = "semver" # javascript, rust, ...

[[file]]
src = "test.txt"
kind = "simple" # strict major.minor.patch format.
```

### Python projects (`pyproject.toml`)

```toml
[project]
version = "1.2.3"

[tool.bver]
git-action = "commit-tag-and-push"

[[tool.bver.file]]
src = "src/mypackage/__init__.py"
kind = "python"
```

### Node.js projects (`package.json`)

```json
{
  "version": "1.2.3",
  "bver": {
    "git-action": "commit-and-tag",
    "file": [
      { "src": "src/version.ts", "kind": "semver" }
    ]
  }
}
```

### Rust projects (`Cargo.toml`)

```toml
[package]
version = "1.2.3"

[package.metadata.bver]
git-action = "commit-and-tag"

[[package.metadata.bver.file]]
src = "src/lib.rs"
kind = "semver"
```

## Version Formats

| Kind | Format | Example |
|------|--------|---------|
| `any` | Any string (no validation) | `v1.2.3-custom` |
| `simple` | `major.minor.patch` | `1.2.3` |
| `python` | PEP 440 | `1.2.3a1.post1.dev1+local` |
| `semver` | Semantic Versioning | `1.2.3-alpha.1+build` |

### Version Casting

When `on-invalid-version = "cast"`, bver automatically converts versions between formats:

- **To simple**: Strips pre-release, post, dev, local, and epoch (`1.2.3a1` -> `1.2.3`)
- **To semver**: Converts Python pre-releases (`1.2.3a1` -> `1.2.3-alpha.1`)
- **To python**: Most formats are already valid PEP 440

## TUI Controls

When bumping versions, an interactive TUI shows proposed changes:

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Navigate changes |
| `Space` | Toggle selection |
| `a` | Select all |
| `n` | Deselect all |
| `Enter` | Apply selected changes |
| `q`/`Esc` | Cancel |

## Git Actions

| Setting | Behavior |
|---------|----------|
| `disabled` | No git operations |
| `commit` | Stage all + commit |
| `commit-and-tag` | Stage all + commit + annotated tag |
| `commit-tag-and-push` | Stage all + commit + tag + push + push tag |

Use `--force` to overwrite existing tags and force push.

## A note on AI

* I just made AI write what I want my version bumper to be. The code in this project
is entirely written by Claude. That being said, human contributions are also welcome 😉.

## License

MIT
