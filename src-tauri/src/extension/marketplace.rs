use std::path::Path;

use serde::Serialize;

use crate::error::ExtensionError;

use super::{registry, ExtensionManifest, ExtensionTheme};

pub fn builtin_theme_toml(theme_identifier: &str) -> Option<&'static str> {
    match theme_identifier {
        "minimal" => Some(include_str!(
            "builtin/keira.theme-minimal/themes/minimal.toml"
        )),
        "midnight" => Some(include_str!(
            "builtin/keira.theme-midnight/themes/midnight.toml"
        )),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct BuiltinExtension {
    manifest: &'static str,
}

pub fn builtin_marketplace(
    extensions_dir: &Path,
) -> Result<Vec<MarketplaceExtension>, ExtensionError> {
    let registry = registry::load_registry(extensions_dir)?;

    Ok(builtin_extensions()
        .iter()
        .map(|builtin| {
            let manifest = parse_builtin_manifest(builtin.manifest);

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
        .iter()
        .any(|builtin| parse_builtin_manifest(builtin.manifest).id == extension_id)
}

fn parse_builtin_manifest(manifest: &str) -> ExtensionManifest {
    toml::from_str(manifest).expect("builtin extension manifests should be valid TOML")
}

fn builtin_extensions() -> &'static [BuiltinExtension] {
    &[
        BuiltinExtension {
            manifest: include_str!("builtin/keira.theme-minimal/extension.toml"),
        },
        BuiltinExtension {
            manifest: include_str!("builtin/keira.theme-midnight/extension.toml"),
        },
    ]
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
