use serde::{Deserialize, Serialize};

/// Directory information for workspace navigation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectoryInfo {
    /// Directory name
    pub name: String,
    /// Full path to the directory
    pub path: String,
}
