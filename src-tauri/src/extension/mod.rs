mod discovery;
mod marketplace;
mod registry;

use std::{fs, path::{Path, PathBuf}};
use std::collections::HashSet;

use crate::error::ExtensionError;
use serde::{Deserialize, Serialize};
use tauri::{Manager, Runtime};
use std::sync::Mutex;

const EXTENSIONS_DIR: &str = "extensions";
const EXTENSIONS_FILE: &str = "extensions.toml";
const EXTENSION_MANIFEST_FILE: &str = "extension.toml";

pub use discovery::list_extensions_from_dir;
pub use marketplace::{builtin_marketplace, is_builtin_extension_id};
pub use registry::{remove_extension_state_in_dir, set_extension_enabled_in_dir};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SearchExtensionsFilter {
    pub query: Option<String>,
    pub installed: Option<bool>,
    pub enabled: Option<bool>,
    pub builtin: Option<bool>,
    pub kind: Option<String>,
    pub identifier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionTheme {
    #[serde(alias = "id")]
    pub identifier: String,
    pub label: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstalledTheme {
    pub identifier: String,
    pub label: String,
    pub path: String,
    pub enabled: bool,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledExtension {
    pub id: String,
    pub publisher: String,
    pub verified: bool,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub kind: String,
    pub enabled: bool,
    pub path: String,
    pub themes: Vec<ExtensionTheme>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadedExtension {
    pub id: String,
    pub identifier: String,
    pub publisher: String,
    pub verified: bool,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub kind: String,
    pub builtin: bool,
    pub installed: bool,
    pub enabled: bool,
    pub path: Option<String>,
    pub themes: Vec<ExtensionTheme>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionSearchResult {
    pub id: String,
    pub identifier: String,
    pub publisher: String,
    pub verified: bool,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub kind: String,
    pub builtin: bool,
    pub installed: bool,
    pub enabled: Option<bool>,
    pub path: Option<String>,
    pub themes: Vec<ExtensionTheme>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ExtensionManifest {
    pub id: String,
    pub publisher: String,
    pub verified: bool,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub kind: String,
    pub themes: Vec<ExtensionTheme>,
}

#[derive(Debug, Default)]
pub struct ExtensionRuntimeState {
    pub loaded_extensions: Mutex<Option<Vec<LoadedExtension>>>,
}

pub fn reset_runtime_state(runtime: &ExtensionRuntimeState) -> Result<(), ExtensionError> {
    let mut state = runtime
        .loaded_extensions
        .lock()
        .map_err(|_| ExtensionError::ReadFile {
            path: PathBuf::from("extension runtime state"),
            source: std::io::Error::other("extension runtime state lock poisoned"),
        })?;
    *state = None;
    Ok(())
}

impl Default for ExtensionManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            publisher: String::new(),
            verified: false,
            name: String::new(),
            version: String::new(),
            description: None,
            kind: "theme".into(),
            themes: Vec::new(),
        }
    }
}

pub fn extensions_dir<R: Runtime, M: Manager<R>>(manager: &M) -> Result<PathBuf, ExtensionError> {
    let extensions_dir = manager
        .path()
        .app_config_dir()
        .map(|dir| dir.join(EXTENSIONS_DIR))
        .map_err(|source| ExtensionError::ResolveExtensionsDir { source })?;

    fs::create_dir_all(&extensions_dir).map_err(|source| ExtensionError::CreateDir {
        path: extensions_dir.clone(),
        source,
    })?;

    Ok(extensions_dir)
}

pub fn extension_registry_path(extensions_dir: &Path) -> PathBuf {
    extensions_dir.join(EXTENSIONS_FILE)
}

pub fn extension_manifest_path(extension_dir: &Path) -> PathBuf {
    extension_dir.join(EXTENSION_MANIFEST_FILE)
}

pub fn list_extensions<R: Runtime, M: Manager<R>>(
    manager: &M,
) -> Result<Vec<InstalledExtension>, ExtensionError> {
    list_extensions_from_dir(&extensions_dir(manager)?)
}

pub fn load_extensions<R: Runtime, M: Manager<R>>(
    manager: &M,
) -> Result<Vec<LoadedExtension>, ExtensionError> {
    let target_dir = extensions_dir(manager)?;
    load_extensions_from_dir(&target_dir)
}

pub fn load_extensions_into_state<R: Runtime, M: Manager<R>>(
    manager: &M,
    runtime: &ExtensionRuntimeState,
) -> Result<Vec<LoadedExtension>, ExtensionError> {
    let loaded = load_extensions(manager)?;
    let mut state = runtime
        .loaded_extensions
        .lock()
        .map_err(|_| ExtensionError::ReadFile {
            path: PathBuf::from("extension runtime state"),
            source: std::io::Error::other("extension runtime state lock poisoned"),
        })?;
    *state = Some(loaded.clone());
    Ok(loaded)
}

pub fn set_extension_enabled<R: Runtime, M: Manager<R>>(
    manager: &M,
    extension_id: &str,
    enabled: bool,
) -> Result<Vec<InstalledExtension>, ExtensionError> {
    let target_dir = extensions_dir(manager)?;
    set_extension_enabled_in_dir(&target_dir, extension_id, enabled)
}

pub fn uninstall_extension<R: Runtime, M: Manager<R>>(
    manager: &M,
    extension_id: &str,
) -> Result<Vec<InstalledExtension>, ExtensionError> {
    let target_dir = extensions_dir(manager)?;
    uninstall_extension_in_dir(&target_dir, extension_id)
}

pub fn search_extensions<R: Runtime, M: Manager<R>>(
    manager: &M,
    filter: SearchExtensionsFilter,
) -> Result<Vec<ExtensionSearchResult>, ExtensionError> {
    let target_dir = extensions_dir(manager)?;
    search_extensions_in_dir(&target_dir, filter)
}

pub fn list_themes_from_state(
    runtime: &ExtensionRuntimeState,
    selected_theme: &str,
) -> Result<Vec<InstalledTheme>, ExtensionError> {
    let state = runtime
        .loaded_extensions
        .lock()
        .map_err(|_| ExtensionError::ReadFile {
            path: PathBuf::from("extension runtime state"),
            source: std::io::Error::other("extension runtime state lock poisoned"),
        })?;

    let Some(extensions) = state.as_ref() else {
        return Ok(Vec::new());
    };

    list_themes_from_loaded(extensions, selected_theme)
}

pub fn resolve_theme_identifier_from_state(
    runtime: &ExtensionRuntimeState,
    identifier: &str,
) -> Result<String, ExtensionError> {
    let state = runtime
        .loaded_extensions
        .lock()
        .map_err(|_| ExtensionError::ReadFile {
            path: PathBuf::from("extension runtime state"),
            source: std::io::Error::other("extension runtime state lock poisoned"),
        })?;

    let Some(extensions) = state.as_ref() else {
        return Err(ExtensionError::ThemeNotFound {
            theme: identifier.to_owned(),
        });
    };

    resolve_theme_identifier_from_loaded(extensions, identifier)
}

pub(crate) fn installed_extension_from_manifest(
    manifest: ExtensionManifest,
    path: &Path,
    enabled: bool,
) -> InstalledExtension {
    InstalledExtension {
        id: manifest.id,
        publisher: manifest.publisher,
        verified: manifest.verified,
        name: manifest.name,
        version: manifest.version,
        description: manifest.description,
        kind: manifest.kind,
        enabled,
        path: path.display().to_string(),
        themes: manifest.themes,
    }
}

fn identifier_from_extension_id(extension_id: &str) -> String {
    extension_id
        .split_once('.')
        .map(|(identifier, _)| identifier)
        .unwrap_or(extension_id)
        .to_owned()
}

pub(crate) fn uninstall_extension_in_dir(
    extensions_dir: &Path,
    extension_id: &str,
) -> Result<Vec<InstalledExtension>, ExtensionError> {
    if is_builtin_extension_id(extension_id) {
        return Err(ExtensionError::BuiltinExtensionCannotBeUninstalled {
            extension_id: extension_id.to_owned(),
        });
    }

    let installed = list_extensions_from_dir(extensions_dir)?;

    let extension = installed
        .iter()
        .find(|item| item.id == extension_id)
        .ok_or_else(|| ExtensionError::ExtensionNotInstalled {
            extension_id: extension_id.to_owned(),
        })?;

    let extension_path = PathBuf::from(&extension.path);
    if extension_path.exists() {
        fs::remove_dir_all(&extension_path).map_err(|source| ExtensionError::RemoveExtensionDir {
            path: extension_path,
            source,
        })?;
    }

    remove_extension_state_in_dir(extensions_dir, extension_id)?;
    list_extensions_from_dir(extensions_dir)
}

pub(crate) fn load_extensions_from_dir(
    extensions_dir: &Path,
) -> Result<Vec<LoadedExtension>, ExtensionError> {
    let installed = list_extensions_from_dir(extensions_dir)?;
    let builtin = builtin_marketplace(extensions_dir)?;
    let mut loaded = Vec::new();

    for extension in builtin {
        if !extension.enabled {
            continue;
        }

        loaded.push(LoadedExtension {
            id: extension.id,
            identifier: extension.identifier,
            publisher: extension.publisher,
            verified: extension.verified,
            name: extension.name,
            version: extension.version,
            description: extension.description,
            kind: extension.kind,
            builtin: true,
            installed: true,
            enabled: extension.enabled,
            path: None,
            themes: extension.themes,
        });
    }

    for extension in installed {
        if !extension.enabled || loaded.iter().any(|item| item.id == extension.id) {
            continue;
        }

        loaded.push(LoadedExtension {
            id: extension.id.clone(),
            identifier: identifier_from_extension_id(&extension.id),
            publisher: extension.publisher,
            verified: extension.verified,
            name: extension.name,
            version: extension.version,
            description: extension.description,
            kind: extension.kind,
            builtin: false,
            installed: true,
            enabled: true,
            path: Some(extension.path),
            themes: extension.themes,
        });
    }

    loaded.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));

    Ok(loaded)
}

pub(crate) fn search_extensions_in_dir(
    extensions_dir: &Path,
    filter: SearchExtensionsFilter,
) -> Result<Vec<ExtensionSearchResult>, ExtensionError> {
    let filter = normalize_search_filter(filter);
    let installed = list_extensions_from_dir(extensions_dir)?;
    let include_builtin_marketplace = filter.builtin == Some(true);
    let mut results = Vec::new();

    if include_builtin_marketplace {
        let marketplace = builtin_marketplace(extensions_dir)?;
        results.extend(marketplace.into_iter().map(|extension| {
            ExtensionSearchResult {
                id: extension.id,
                identifier: extension.identifier,
                publisher: extension.publisher,
                verified: extension.verified,
                name: extension.name,
                version: extension.version,
                description: extension.description,
                kind: extension.kind,
                builtin: extension.builtin,
                installed: extension.installed,
                enabled: Some(extension.enabled),
                path: None,
                themes: extension.themes,
            }
        }));
    }

    for extension in installed {
        if results.iter().any(|item| item.id == extension.id) {
            continue;
        }

        let identifier = identifier_from_extension_id(&extension.id);

        results.push(ExtensionSearchResult {
            id: extension.id,
            identifier,
            publisher: extension.publisher,
            verified: extension.verified,
            name: extension.name,
            version: extension.version,
            description: extension.description,
            kind: extension.kind,
            builtin: false,
            installed: true,
            enabled: Some(extension.enabled),
            path: Some(extension.path),
            themes: extension.themes,
        });
    }

    let query = filter.query.as_deref().map(str::trim).filter(|value| !value.is_empty()).map(|value| value.to_lowercase());

    results.retain(|extension| matches_extension(extension, &filter, query.as_deref()));
    results.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));

    Ok(results)
}

pub(crate) fn list_themes_from_dir(
    extensions_dir: &Path,
    selected_theme: &str,
) -> Result<Vec<InstalledTheme>, ExtensionError> {
    let extensions = load_extensions_from_dir(extensions_dir)?;
    list_themes_from_loaded(&extensions, selected_theme)
}

fn list_themes_from_loaded(
    extensions: &[LoadedExtension],
    selected_theme: &str,
) -> Result<Vec<InstalledTheme>, ExtensionError> {
    let mut seen_identifiers = HashSet::new();
    let mut themes = Vec::new();

    for extension in extensions {
        for theme in &extension.themes {
            if !seen_identifiers.insert(theme.identifier.clone()) {
                return Err(ExtensionError::DuplicateThemeIdentifier {
                    identifier: theme.identifier.clone(),
                });
            }

            let selected = theme.identifier == selected_theme;
            themes.push(InstalledTheme {
                identifier: theme.identifier.clone(),
                label: theme.label.clone(),
                path: theme.path.clone(),
                enabled: extension.enabled,
                selected,
            });
        }
    }

    themes.sort_by(|left, right| left.label.cmp(&right.label).then(left.path.cmp(&right.path)));

    Ok(themes)
}

pub(crate) fn resolve_theme_identifier_from_dir(
    extensions_dir: &Path,
    identifier: &str,
) -> Result<String, ExtensionError> {
    let extensions = load_extensions_from_dir(extensions_dir)?;
    resolve_theme_identifier_from_loaded(&extensions, identifier)
}

fn resolve_theme_identifier_from_loaded(
    extensions: &[LoadedExtension],
    identifier: &str,
) -> Result<String, ExtensionError> {
    let mut seen_identifiers = HashSet::new();

    for extension in extensions {
        for extension_theme in &extension.themes {
            if !seen_identifiers.insert(extension_theme.identifier.clone()) {
                return Err(ExtensionError::DuplicateThemeIdentifier {
                    identifier: extension_theme.identifier.clone(),
                });
            }

            if extension_theme.identifier == identifier {
                return Ok(extension_theme.identifier.clone());
            }
        }
    }

    Err(ExtensionError::ThemeNotFound {
        theme: identifier.to_owned(),
    })
}

fn matches_extension(
    extension: &ExtensionSearchResult,
    filter: &SearchExtensionsFilter,
    query: Option<&str>,
) -> bool {
    if let Some(query) = query {
        let description = extension.description.as_deref().unwrap_or_default().to_lowercase();
        let name = extension.name.to_lowercase();
        let id = extension.id.to_lowercase();
        let identifier = extension.identifier.to_lowercase();

        if !name.contains(query)
            && !id.contains(query)
            && !identifier.contains(query)
            && !description.contains(query)
        {
            return false;
        }
    }

    if let Some(installed) = filter.installed {
        if extension.installed != installed {
            return false;
        }
    }

    if let Some(enabled) = filter.enabled {
        if extension.enabled != Some(enabled) {
            return false;
        }
    }

    if let Some(builtin) = filter.builtin {
        if extension.builtin != builtin {
            return false;
        }
    }

    if let Some(kind) = filter.kind.as_deref() {
        if extension.kind != kind {
            return false;
        }
    }

    if let Some(identifier) = filter.identifier.as_deref() {
        if extension.identifier != identifier {
            return false;
        }
    }

    true
}

fn normalize_search_filter(mut filter: SearchExtensionsFilter) -> SearchExtensionsFilter {
    let Some(query) = filter.query.take() else {
        return filter;
    };

    let mut text_terms = Vec::new();

    for term in query.split_whitespace() {
        if !apply_query_token(term, &mut filter) {
            text_terms.push(term);
        }
    }

    filter.query = if text_terms.is_empty() {
        None
    } else {
        Some(text_terms.join(" "))
    };

    filter
}

fn apply_query_token(term: &str, filter: &mut SearchExtensionsFilter) -> bool {
    match term {
        "@installed" => {
            filter.installed = Some(true);
            true
        }
        "@builtin" => {
            filter.builtin = Some(true);
            true
        }
        "@enabled" => {
            filter.enabled = Some(true);
            true
        }
        "@disabled" => {
            filter.enabled = Some(false);
            true
        }
        _ => {
            if let Some(value) = term.strip_prefix("@kind:") {
                if !value.is_empty() {
                    filter.kind = Some(value.to_owned());
                    return true;
                }
            }

            if let Some(value) = term.strip_prefix("@identifier:") {
                if !value.is_empty() {
                    filter.identifier = Some(value.to_owned());
                    return true;
                }
            }

            if let Some(value) = term.strip_prefix("@publisher:") {
                if !value.is_empty() {
                    filter.identifier = Some(value.to_owned());
                    return true;
                }
            }

            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        builtin_marketplace, list_extensions_from_dir,
        list_themes_from_dir, load_extensions_from_dir, resolve_theme_identifier_from_dir,
        search_extensions_in_dir,
        set_extension_enabled_in_dir, uninstall_extension_in_dir,
        SearchExtensionsFilter,
    };
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("keira4-extensions-test-{nanos}"))
    }

    fn write_extension(dir: &PathBuf, id: &str, theme_name: &str) {
        let extension_dir = dir.join(id);
        let publisher = id.split('.').next().unwrap_or(id);
        fs::create_dir_all(extension_dir.join("themes"))
            .expect("extension themes directory should be created");
        fs::write(
            extension_dir.join("extension.toml"),
            format!(
                "id = \"{id}\"\npublisher = \"{publisher}\"\nverified = false\nname = \"{id}\"\nversion = \"1.0.0\"\nkind = \"theme\"\n[[themes]]\nid = \"{id}.dark\"\nlabel = \"{theme_name}\"\npath = \"themes/dark.json\"\n"
            ),
        )
        .expect("extension manifest should be written");
        fs::write(extension_dir.join("themes/dark.json"), "{}")
            .expect("theme file should be written");
    }

    #[test]
    fn loads_installed_extensions_from_directory() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");
        write_extension(&dir, "publisher.theme-one", "Theme One");

        let extensions = list_extensions_from_dir(&dir)
            .expect("extensions should load from directory");

        assert_eq!(extensions.len(), 1);
        assert_eq!(extensions[0].id, "publisher.theme-one");
        assert_eq!(extensions[0].publisher, "publisher");
        assert!(!extensions[0].verified);
        assert!(extensions[0].enabled);
        assert_eq!(extensions[0].themes.len(), 1);

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn applies_enabled_state_from_registry_file() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");
        write_extension(&dir, "publisher.theme-one", "Theme One");
        fs::write(
            dir.join("extensions.toml"),
            "[extensions.\"publisher.theme-one\"]\nenabled = false\n",
        )
        .expect("extensions registry should be written");

        let extensions = list_extensions_from_dir(&dir)
            .expect("extensions should load from directory");

        assert_eq!(extensions[0].id, "publisher.theme-one");
        assert!(!extensions[0].enabled);

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn updates_extension_enabled_state() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");
        write_extension(&dir, "publisher.theme-one", "Theme One");

        let extensions = set_extension_enabled_in_dir(&dir, "publisher.theme-one", false)
            .expect("enabled state should be updated");

        assert_eq!(extensions.len(), 1);
        assert!(!extensions[0].enabled);
        let registry = fs::read_to_string(dir.join("extensions.toml"))
            .expect("extensions registry should exist after update");
        assert!(registry.contains("enabled = false"));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn loads_builtin_marketplace_entries() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");

        let marketplace = builtin_marketplace(&dir)
            .expect("builtin marketplace should load");

        assert_eq!(marketplace.len(), 2);
        assert!(marketplace.iter().all(|item| item.builtin));
        assert!(marketplace.iter().all(|item| item.installed));
        assert!(marketplace.iter().all(|item| item.enabled));
        assert!(marketplace.iter().all(|item| item.publisher == "keira"));
        assert!(marketplace.iter().all(|item| item.verified));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn uninstalls_extension_and_cleans_registry() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");
        write_extension(&dir, "publisher.theme-one", "Theme One");
        fs::write(
            dir.join("extensions.toml"),
            "[extensions.\"publisher.theme-one\"]\nenabled = false\n",
        )
        .expect("extensions registry should be written");

        let remaining = uninstall_extension_in_dir(&dir, "publisher.theme-one")
            .expect("extension should uninstall");

        assert!(remaining.is_empty());
        assert!(!dir.join("publisher.theme-one").exists());
        assert!(!dir.join("extensions.toml").exists());

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn returns_error_when_uninstalling_missing_extension() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");

        let error = uninstall_extension_in_dir(&dir, "publisher.missing-extension")
            .expect_err("missing extension uninstall should fail");

        assert!(error.to_string().contains("is not installed"));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn searches_extensions_by_name_and_filters() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");
        write_extension(&dir, "publisher.theme-local", "Local Theme");
        fs::write(
            dir.join("extensions.toml"),
            "[extensions.\"publisher.theme-local\"]\nenabled = false\n",
        )
        .expect("extensions registry should be written");

        let local_results = search_extensions_in_dir(
            &dir,
            SearchExtensionsFilter {
                query: Some("local".into()),
                installed: Some(true),
                enabled: Some(false),
                builtin: Some(false),
                kind: Some("theme".into()),
                identifier: None,
            },
        )
        .expect("extension search should succeed");

        assert_eq!(local_results.len(), 1);
        assert_eq!(local_results[0].id, "publisher.theme-local");
        assert_eq!(local_results[0].publisher, "publisher");
        assert!(!local_results[0].verified);

        let builtin_results = search_extensions_in_dir(
            &dir,
            SearchExtensionsFilter {
                query: Some("minimal".into()),
                installed: Some(true),
                enabled: Some(true),
                builtin: Some(true),
                kind: Some("theme".into()),
                identifier: None,
            },
        )
        .expect("builtin extension search should succeed");

        assert_eq!(builtin_results.len(), 1);
        assert_eq!(builtin_results[0].id, "keira.theme-minimal");
        assert_eq!(builtin_results[0].publisher, "keira");
        assert!(builtin_results[0].verified);

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn searches_extensions_with_query_tokens() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");
        write_extension(&dir, "publisher.theme-local", "Local Theme");
        fs::write(
            dir.join("extensions.toml"),
            "[extensions.\"publisher.theme-local\"]\nenabled = false\n",
        )
        .expect("extensions registry should be written");

        let local_results = search_extensions_in_dir(
            &dir,
            SearchExtensionsFilter {
                query: Some("@installed @disabled @identifier:publisher local".into()),
                ..Default::default()
            },
        )
        .expect("query token search should succeed");

        assert_eq!(local_results.len(), 1);
        assert_eq!(local_results[0].id, "publisher.theme-local");
        assert_eq!(local_results[0].identifier, "publisher");
        assert_eq!(local_results[0].publisher, "publisher");

        let builtin_results = search_extensions_in_dir(
            &dir,
            SearchExtensionsFilter {
                query: Some("@builtin @enabled @kind:theme @identifier:keira minimal".into()),
                ..Default::default()
            },
        )
        .expect("builtin query token search should succeed");

        assert_eq!(builtin_results.len(), 1);
        assert_eq!(builtin_results[0].id, "keira.theme-minimal");
        assert_eq!(builtin_results[0].identifier, "keira");
        assert_eq!(builtin_results[0].publisher, "keira");
        assert!(builtin_results[0].verified);

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn refuses_to_uninstall_builtin_extension() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");

        let error = uninstall_extension_in_dir(&dir, "keira.theme-minimal")
            .expect_err("builtin extension uninstall should fail");

        assert!(error.to_string().contains("cannot be uninstalled"));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn returns_only_installed_extensions_without_query() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");
        write_extension(&dir, "publisher.theme-local", "Local Theme");

        let results = search_extensions_in_dir(&dir, SearchExtensionsFilter::default())
            .expect("default search should succeed");

        assert_eq!(results.len(), 1);
        assert!(results.iter().all(|item| item.installed));
        assert!(results.iter().all(|item| item.path.is_some()));
        assert!(!results.iter().any(|item| item.id == "keira.theme-midnight"));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn returns_marketplace_and_installed_extensions_with_query() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");

        let results = search_extensions_in_dir(
            &dir,
            SearchExtensionsFilter {
                query: Some("@builtin theme".into()),
                ..Default::default()
            },
        )
        .expect("search with query should succeed");

        assert!(results.iter().any(|item| item.id == "keira.theme-minimal" && item.installed));
        assert!(results.iter().any(|item| item.id == "keira.theme-midnight" && item.installed));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn does_not_include_builtin_extensions_without_builtin_filter() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");

        let results = search_extensions_in_dir(
            &dir,
            SearchExtensionsFilter {
                query: Some("theme".into()),
                ..Default::default()
            },
        )
        .expect("search without builtin filter should succeed");

        assert!(!results.iter().any(|item| item.id == "keira.theme-minimal"));
        assert!(!results.iter().any(|item| item.id == "keira.theme-midnight"));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn lists_themes_and_marks_selected_theme() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");
        write_extension(&dir, "publisher.theme-local", "Local Theme");

        let themes = list_themes_from_dir(&dir, "minimal")
            .expect("themes should load from runtime extensions");

        assert_eq!(themes.len(), 3);
        assert!(themes.iter().any(|theme| theme.label == "Minimal" && theme.selected));
        assert!(themes.iter().any(|theme| theme.label == "Local Theme" && !theme.selected));
        assert!(themes.iter().any(|theme| theme.label == "Midnight" && !theme.selected));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn resolves_theme_by_id_only() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");
        write_extension(&dir, "publisher.theme-local", "Local Theme");

        let by_id = resolve_theme_identifier_from_dir(&dir, "publisher.theme-local.dark")
            .expect("theme id should resolve");
        let by_label = resolve_theme_identifier_from_dir(&dir, "Local Theme")
            .expect_err("theme label should not resolve");

        assert_eq!(by_id, "publisher.theme-local.dark");
        assert!(by_label.to_string().contains("was not found"));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn loads_enabled_installed_extensions_and_builtin_extensions() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");
        write_extension(&dir, "publisher.theme-local", "Local Theme");

        let loaded = load_extensions_from_dir(&dir)
            .expect("enabled installed and builtin extensions should load");

        assert!(loaded.iter().any(|extension| extension.id == "publisher.theme-local" && extension.enabled));
        assert!(loaded.iter().any(|extension| extension.id == "keira.theme-minimal" && extension.builtin));
        assert!(loaded.iter().any(|extension| extension.id == "keira.theme-midnight" && extension.builtin && extension.installed));
        assert!(loaded.iter().any(|extension| extension.id == "keira.theme-minimal" && extension.publisher == "keira" && extension.verified));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn skips_disabled_extensions_when_loading_runtime_extensions() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");
        write_extension(&dir, "publisher.theme-local", "Local Theme");
        set_extension_enabled_in_dir(&dir, "publisher.theme-local", false)
            .expect("local extension should be disabled");
        set_extension_enabled_in_dir(&dir, "keira.theme-minimal", false)
            .expect("builtin extension should be disabled");

        let loaded = load_extensions_from_dir(&dir)
            .expect("disabled extensions should be skipped while loading");

        assert!(!loaded.iter().any(|extension| extension.id == "publisher.theme-local"));
        assert!(!loaded.iter().any(|extension| extension.id == "keira.theme-minimal"));
        assert!(loaded.iter().any(|extension| extension.id == "keira.theme-midnight"));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }

    #[test]
    fn returns_error_for_duplicate_theme_identifier() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("extensions directory should be created");

        let first = dir.join("publisher.theme-one");
        fs::create_dir_all(first.join("themes")).expect("first extension directory should be created");
        fs::write(
            first.join("extension.toml"),
            "id = \"publisher.theme-one\"\npublisher = \"publisher\"\nverified = false\nname = \"Theme One\"\nversion = \"1.0.0\"\nkind = \"theme\"\n[[themes]]\nid = \"duplicate\"\nlabel = \"Duplicate One\"\npath = \"themes/one.json\"\n",
        )
        .expect("first extension manifest should be written");
        fs::write(first.join("themes/one.json"), "{}").expect("first theme file should be written");

        let second = dir.join("publisher.theme-two");
        fs::create_dir_all(second.join("themes")).expect("second extension directory should be created");
        fs::write(
            second.join("extension.toml"),
            "id = \"publisher.theme-two\"\npublisher = \"publisher\"\nverified = false\nname = \"Theme Two\"\nversion = \"1.0.0\"\nkind = \"theme\"\n[[themes]]\nid = \"duplicate\"\nlabel = \"Duplicate Two\"\npath = \"themes/two.json\"\n",
        )
        .expect("second extension manifest should be written");
        fs::write(second.join("themes/two.json"), "{}").expect("second theme file should be written");

        let error = list_themes_from_dir(&dir, "duplicate")
            .expect_err("duplicate identifiers should fail");

        assert!(error.to_string().contains("is duplicated"));

        fs::remove_dir_all(&dir).expect("extensions directory should be removed");
    }
}
