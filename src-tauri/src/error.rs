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

#[derive(Debug, Error)]
pub enum ExtensionError {
    #[error("could not determine the app extensions directory: {source}")]
    ResolveExtensionsDir {
        #[source]
        source: tauri::Error,
    },
    #[error("could not read directory at {path}: {source}")]
    ReadDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not read directory entry in {path}: {source}")]
    ReadDirEntry {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid extension manifest at {path}: {message}")]
    InvalidExtensionManifest { path: PathBuf, message: String },
    #[error("could not remove extension directory at {path}: {source}")]
    RemoveExtensionDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not create directory at {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not read file at {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid TOML in file at {path}: {source}")]
    ParseToml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("could not serialize TOML: {source}")]
    SerializeToml {
        #[source]
        source: toml::ser::Error,
    },
    #[error("could not write file at {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("builtin extension `{extension_id}` cannot be uninstalled")]
    BuiltinExtensionCannotBeUninstalled { extension_id: String },
    #[error("extension `{extension_id}` is not installed")]
    ExtensionNotInstalled { extension_id: String },
    #[error("theme identifier `{identifier}` is duplicated")]
    DuplicateThemeIdentifier { identifier: String },
    #[error("theme `{theme}` was not found")]
    ThemeNotFound { theme: String },
}
