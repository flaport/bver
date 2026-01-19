use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub current_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_lines: Option<usize>,
    #[serde(default)]
    pub default_kind: FileKind,
    #[serde(default)]
    pub on_invalid_version: OnInvalidVersion,
    #[serde(default)]
    pub run_pre_commit: RunPreCommit,
    #[serde(default)]
    pub git_action: GitAction,
    #[serde(default, rename = "file")]
    pub files: Vec<FileConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileConfig {
    pub src: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<FileKind>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    #[default]
    Any,
    Simple,
    Python,
    Semver,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OnInvalidVersion {
    #[default]
    Error,
    Cast,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RunPreCommit {
    Enabled,
    Disabled,
    #[default]
    WhenPresent,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GitAction {
    Disabled,
    Commit,
    #[default]
    CommitAndTag,
    CommitTagAndPush,
}
