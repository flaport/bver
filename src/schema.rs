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
    pub git: GitConfig,
    #[serde(default, rename = "file")]
    pub files: Vec<FileConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct GitConfig {
    #[serde(default = "default_actions")]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub run_pre_commit: RunPreCommit,
    #[serde(default = "default_tag_template")]
    pub tag_template: String,
    #[serde(default = "default_commit_template")]
    pub commit_template: String,
    #[serde(default = "default_branch_template")]
    pub branch_template: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            actions: default_actions(),
            run_pre_commit: RunPreCommit::default(),
            tag_template: default_tag_template(),
            commit_template: default_commit_template(),
            branch_template: default_branch_template(),
        }
    }
}

impl GitConfig {
    pub fn has(&self, action: Action) -> bool {
        self.actions.contains(&action)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.has(Action::Tag) && !self.has(Action::Commit) {
            return Err("git action 'tag' requires 'commit'".to_string());
        }
        if self.has(Action::Push) && !self.has(Action::Commit) {
            return Err("git action 'push' requires 'commit'".to_string());
        }
        if self.has(Action::Pr) && !self.has(Action::Push) {
            return Err("git action 'pr' requires 'push'".to_string());
        }
        if self.has(Action::Pr) && !self.has(Action::Branch) {
            return Err("git action 'pr' requires 'branch'".to_string());
        }
        if self.has(Action::Tag) && self.has(Action::Branch) {
            return Err("git actions 'tag' and 'branch' cannot coexist".to_string());
        }
        Ok(())
    }
}

fn default_actions() -> Vec<Action> {
    vec![Action::AddAll, Action::Commit, Action::Tag]
}

fn default_tag_template() -> String {
    "{new-version}".to_string()
}

fn default_commit_template() -> String {
    "Bump version from {current-version} to {new-version}".to_string()
}

fn default_branch_template() -> String {
    "release/{new-version}".to_string()
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    AddAll,
    Branch,
    Commit,
    Tag,
    Push,
    Pr,
}
