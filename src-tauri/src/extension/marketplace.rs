use std::{fs, path::{Component, Path, PathBuf}};

use serde::Serialize;

use crate::error::ExtensionError;

use super::{registry, ExtensionManifest, ExtensionTheme};

#[derive(Debug, Clone)]
struct BuiltinExtension {
    path: PathBuf,
    manifest: ExtensionManifest,
}

pub fn builtin_theme_toml(theme_identifier: &str) -> Result<Option<String>, ExtensionError> {
    for builtin in builtin_extensions()? {
        for theme in &builtin.manifest.themes {
            if theme.identifier != theme_identifier {
                continue;
            }

            let theme_path = resolve_builtin_file_path(&builtin.path, &theme.path)?;
            let content = fs::read_to_string(&theme_path).map_err(|source| ExtensionError::ReadFile {
                path: theme_path,
                source,
            })?;

            return Ok(Some(content));
        }
    }

    Ok(None)
}

pub fn builtin_marketplace(
    extensions_dir: &Path,
) -> Result<Vec<MarketplaceExtension>, ExtensionError> {
    let registry = registry::load_registry(extensions_dir)?;

    Ok(builtin_extensions()?
        .into_iter()
        .map(|builtin| {
            let manifest = builtin.manifest;

            MarketplaceExtension {
                id: manifest.id.clone(),
                identifier: manifest
                    .id
                    .split_once('.')
                    .map(|(identifier, _)| identifier)
                    .unwrap_or(&manifest.id)
                    .to_owned(),
                publisher: manifest.publisher.clone(),
                verified: manifest.verified,
                name: manifest.name.clone(),
                version: manifest.version.clone(),
                description: manifest.description.clone(),
                kind: manifest.kind.clone(),
                themes: manifest.themes.clone(),
                builtin: true,
                installed: true,
                enabled: registry.enabled_for(&manifest.id),
            }
        })
        .collect())
}

pub fn is_builtin_extension_id(extension_id: &str) -> bool {
    builtin_extensions()
        .map(|extensions| extensions.into_iter().any(|builtin| builtin.manifest.id == extension_id))
        .unwrap_or(false)
}

fn builtin_extensions() -> Result<Vec<BuiltinExtension>, ExtensionError> {
    let dir = builtin_extensions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut extensions = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|source| ExtensionError::ReadDir {
        path: dir.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| ExtensionError::ReadDirEntry {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("extension.toml");
        let content = fs::read_to_string(&manifest_path).map_err(|source| ExtensionError::ReadFile {
            path: manifest_path.clone(),
            source,
        })?;
        let manifest: ExtensionManifest = toml::from_str(&content).map_err(|source| ExtensionError::ParseToml {
            path: manifest_path.clone(),
            source,
        })?;

        validate_builtin_manifest(&manifest_path, &manifest)?;
        extensions.push(BuiltinExtension { path, manifest });
    }

    extensions.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    Ok(extensions)
}

fn builtin_extensions_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("extension")
        .join("builtin")
}

fn validate_builtin_manifest(
    manifest_path: &Path,
    manifest: &ExtensionManifest,
) -> Result<(), ExtensionError> {
    if manifest.id.trim().is_empty()
        || manifest.name.trim().is_empty()
        || manifest.version.trim().is_empty()
    {
        return Err(ExtensionError::InvalidExtensionManifest {
            path: manifest_path.to_path_buf(),
            message: "`id`, `name` and `version` are required".into(),
        });
    }

    for theme in &manifest.themes {
        validate_relative_path(manifest_path, &theme.path)?;
    }

    Ok(())
}

fn validate_relative_path(manifest_path: &Path, relative_path: &str) -> Result<(), ExtensionError> {
    let path = Path::new(relative_path);
    let unsafe_path = path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)));

    if unsafe_path {
        return Err(ExtensionError::InvalidExtensionManifest {
            path: manifest_path.to_path_buf(),
            message: format!("theme path `{relative_path}` must stay inside the extension directory"),
        });
    }

    Ok(())
}

fn resolve_builtin_file_path(
    extension_path: &Path,
    relative_path: &str,
) -> Result<PathBuf, ExtensionError> {
    validate_relative_path(&extension_path.join("extension.toml"), relative_path)?;

    let base = extension_path.canonicalize().map_err(|source| ExtensionError::ReadFile {
        path: extension_path.to_path_buf(),
        source,
    })?;
    let candidate = base.join(relative_path);
    let resolved = candidate.canonicalize().map_err(|source| ExtensionError::ReadFile {
        path: candidate.clone(),
        source,
    })?;

    if !resolved.starts_with(&base) {
        return Err(ExtensionError::InvalidExtensionManifest {
            path: candidate,
            message: "theme path resolves outside the builtin extension directory".into(),
        });
    }

    Ok(resolved)
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceExtension {
    pub id: String,
    pub identifier: String,
    pub publisher: String,
    pub verified: bool,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub kind: String,
    pub themes: Vec<ExtensionTheme>,
    pub builtin: bool,
    pub installed: bool,
    pub enabled: bool,
}
