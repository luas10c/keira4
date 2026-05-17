use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::ExtensionError;

use super::{extension_registry_path, list_extensions_from_dir};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ExtensionsRegistry {
    extensions: BTreeMap<String, RegistryEntry>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
struct RegistryEntry {
    enabled: bool,
}

impl ExtensionsRegistry {
    pub(crate) fn enabled_for(&self, extension_id: &str) -> bool {
        self.extensions
            .get(extension_id)
            .map(|entry| entry.enabled)
            .unwrap_or(true)
    }

    fn remove(&mut self, extension_id: &str) {
        self.extensions.remove(extension_id);
    }

    fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }
}

pub fn load_registry(extensions_dir: &Path) -> Result<ExtensionsRegistry, ExtensionError> {
    let registry_path = extension_registry_path(extensions_dir);
    if !registry_path.exists() {
        return Ok(ExtensionsRegistry::default());
    }

    let content = fs::read_to_string(&registry_path).map_err(|source| ExtensionError::ReadFile {
        path: registry_path.clone(),
        source,
    })?;

    toml::from_str(&content).map_err(|source| ExtensionError::ParseToml {
        path: registry_path,
        source,
    })
}

pub fn set_extension_enabled_in_dir(
    extensions_dir: &Path,
    extension_id: &str,
    enabled: bool,
) -> Result<Vec<super::InstalledExtension>, ExtensionError> {
    fs::create_dir_all(extensions_dir).map_err(|source| ExtensionError::CreateDir {
        path: extensions_dir.to_path_buf(),
        source,
    })?;

    let mut registry = load_registry(extensions_dir)?;
    registry
        .extensions
        .insert(extension_id.to_owned(), RegistryEntry { enabled });

    let registry_path = extension_registry_path(extensions_dir);
    let content = toml::to_string_pretty(&registry)
        .map_err(|source| ExtensionError::SerializeToml { source })?;
    fs::write(&registry_path, content).map_err(|source| ExtensionError::WriteFile {
        path: registry_path,
        source,
    })?;

    list_extensions_from_dir(extensions_dir)
}

pub fn remove_extension_state_in_dir(
    extensions_dir: &Path,
    extension_id: &str,
) -> Result<(), ExtensionError> {
    let registry_path = extension_registry_path(extensions_dir);
    if !registry_path.exists() {
        return Ok(());
    }

    let mut registry = load_registry(extensions_dir)?;
    registry.remove(extension_id);

    if registry.is_empty() {
        fs::remove_file(&registry_path).map_err(|source| ExtensionError::WriteFile {
            path: registry_path,
            source,
        })?;
        return Ok(());
    }

    let content = toml::to_string_pretty(&registry)
        .map_err(|source| ExtensionError::SerializeToml { source })?;
    fs::write(&registry_path, content).map_err(|source| ExtensionError::WriteFile {
        path: registry_path,
        source,
    })
}
