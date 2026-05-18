use std::{fs, path::{Component, Path}};

use crate::error::ExtensionError;

use super::{
    extension_manifest_path, installed_extension_from_manifest, registry, ExtensionManifest,
    InstalledExtension,
};

pub fn list_extensions_from_dir(
    extensions_dir: &Path,
) -> Result<Vec<InstalledExtension>, ExtensionError> {
    if !extensions_dir.exists() {
        return Ok(Vec::new());
    }

    let registry = registry::load_registry(extensions_dir)?;
    let mut extensions = Vec::new();

    for entry in fs::read_dir(extensions_dir).map_err(|source| ExtensionError::ReadDir {
        path: extensions_dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| ExtensionError::ReadDirEntry {
            path: extensions_dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest = load_manifest(&path)?;
        let enabled = registry.enabled_for(&manifest.id);

        extensions.push(installed_extension_from_manifest(manifest, &path, enabled));
    }

    extensions.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(extensions)
}

fn load_manifest(extension_dir: &Path) -> Result<ExtensionManifest, ExtensionError> {
    let manifest_path = extension_manifest_path(extension_dir);
    let content = fs::read_to_string(&manifest_path).map_err(|source| ExtensionError::ReadFile {
        path: manifest_path.clone(),
        source,
    })?;
    let manifest: ExtensionManifest =
        toml::from_str(&content).map_err(|source| ExtensionError::ParseToml {
            path: manifest_path.clone(),
            source,
        })?;

    if manifest.id.trim().is_empty()
        || manifest.name.trim().is_empty()
        || manifest.version.trim().is_empty()
    {
        return Err(ExtensionError::InvalidExtensionManifest {
            path: manifest_path,
            message: "`id`, `name` and `version` are required".into(),
        });
    }

    for theme in &manifest.themes {
        let theme_path = Path::new(&theme.path);
        let unsafe_path = theme_path.is_absolute()
            || theme_path
                .components()
                .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)));

        if unsafe_path {
            return Err(ExtensionError::InvalidExtensionManifest {
                path: manifest_path,
                message: format!("theme path `{}` must stay inside the extension directory", theme.path),
            });
        }
    }

    Ok(manifest)
}
