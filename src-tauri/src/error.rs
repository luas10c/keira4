use std::path::PathBuf;

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "detail")]
pub enum SshError {
    #[error("SSH authentication failed for user '{user}' at {host}:{port}")]
    AuthFailed { user: String, host: String, port: u16 },
    #[error("SSH host unreachable: {host}:{port} — {reason}")]
    HostUnreachable { host: String, port: u16, reason: String },
    #[error("SSH host key mismatch for {host} — connection refused for safety")]
    HostKeyMismatch { host: String },
    #[error("SSH private key not found: {path}")]
    KeyFileNotFound { path: String },
    #[error("SSH port forward refused: {local_port} -> {remote_host}:{remote_port}")]
    PortForwardFailed {
        local_port: u16,
        remote_host: String,
        remote_port: u16,
    },
    #[error("SSH error: {0}")]
    Io(String),
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Not connected to any database")]
    NotConnected,
    #[error("Already connected. Disconnect first.")]
    AlreadyConnected,
    #[error("MySQL error: {0}")]
    MySql(#[from] mysql::Error),
    #[error("Invalid value in column '{column}': {details}")]
    ValueConversion { column: String, details: String },
    #[error("Invalid identifier '{0}': only alphanumeric and underscore allowed")]
    InvalidIdentifier(String),
    #[error("Cannot order by column '{0}' because it is not included in the SELECT list")]
    MissingOrderByColumn(String),
    #[error("SSH tunnel error: {0}")]
    SshTunnel(#[from] SshError),
}

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
