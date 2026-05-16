use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not determine the app config directory: {source}")]
    ResolveConfigDir {
        #[source]
        source: tauri::Error,
    },
    #[error("could not read config file at {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid TOML in config file at {path}: {source}")]
    ParseToml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid patch value for key `{key}`")]
    InvalidPatchValue { key: String },
    #[error("could not create config directory at {path}: {source}")]
    CreateConfigDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not serialize config to TOML: {source}")]
    SerializeToml {
        #[source]
        source: toml::ser::Error,
    },
    #[error("could not write config file at {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
