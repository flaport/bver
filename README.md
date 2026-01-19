# vrsn

A version management tool.

## Configuration

Configuration can be specified in any of the following files (in order of priority):

| Source           | Location                  |
| ---------------- | ------------------------- |
| `vrsn.toml`      | Root level                |
| `pyproject.toml` | `[tool.vrsn]`             |
| `package.json`   | `"vrsn": {...}`           |
| `Cargo.toml`     | `[package.metadata.vrsn]` |

### Example

```toml
# vrsn.toml (or [tool.vrsn] in pyproject.toml)

[[file]]
src = "src/version.txt"
```
