use std::io;

use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),

    #[error("CLI error: {0}")]
    Cli(String),

    #[error("Invalid YAML path '{path}': {reason}")]
    InvalidPath { path: String, reason: String },

    #[error("Failed to read file '{path}': {source}")]
    ReadFile {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Failed to write file '{path}': {source}")]
    WriteFile {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Failed to read directory '{path}': {source}")]
    ReadDir {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Failed to read directory entry: {0}")]
    ReadDirEntry(#[source] io::Error),

    #[error("Failed to read from stdin: {0}")]
    ReadStdin(#[source] io::Error),

    #[error("Failed to parse YAML {context}: {source}")]
    ParseYaml {
        context: String,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("Invalid regex pattern: {0}")]
    Regex(#[from] regex::Error),

    #[error("Failed to apply YAML patch: {0}")]
    Patch(String),
}

impl AppError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    pub fn cli(message: impl Into<String>) -> Self {
        Self::Cli(message.into())
    }

    pub fn invalid_path(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidPath {
            path: path.into(),
            reason: reason.into(),
        }
    }

    pub fn parse_yaml(context: impl Into<String>, source: serde_yaml::Error) -> Self {
        Self::ParseYaml {
            context: context.into(),
            source,
        }
    }

    pub fn read_file(path: impl Into<String>, source: io::Error) -> Self {
        Self::ReadFile {
            path: path.into(),
            source,
        }
    }

    pub fn write_file(path: impl Into<String>, source: io::Error) -> Self {
        Self::WriteFile {
            path: path.into(),
            source,
        }
    }

    pub fn read_dir(path: impl Into<String>, source: io::Error) -> Self {
        Self::ReadDir {
            path: path.into(),
            source,
        }
    }

    pub fn patch(source: impl Into<String>) -> Self {
        Self::Patch(source.into())
    }
}
