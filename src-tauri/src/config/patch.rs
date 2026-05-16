use std::{fs, path::Path};

use serde_json::Value as JsonValue;
use toml::{Table, Value};

use crate::error::ConfigError;

use super::{AppConfig, ConfigPatch};

pub fn patch_from_path(
    path: &Path,
    patches: &[ConfigPatch],
) -> Result<AppConfig, ConfigError> {
    let mut table = read_table_from_path(path)?;

    for patch in patches {
        let value = toml_value_from_json(&patch.key, &patch.value)?;
        apply_patch(&mut table, &patch.key, value);
    }

    let config = AppConfig::from_table(&table);
    write_to_path(path, &config)?;

    Ok(config)
}

fn read_table_from_path(path: &Path) -> Result<Table, ConfigError> {
    if !path.exists() {
        return Ok(Table::new());
    }

    let content =
        fs::read_to_string(path).map_err(|source| ConfigError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;

    let value: Value =
        toml::from_str(&content).map_err(|source| ConfigError::ParseToml {
            path: path.to_path_buf(),
            source,
        })?;

    Ok(value.as_table().cloned().unwrap_or_default())
}

fn write_to_path(path: &Path, config: &AppConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            ConfigError::CreateConfigDir {
                path: parent.to_path_buf(),
                source,
            }
        })?;
    }

    let content = toml::to_string_pretty(config)
        .map_err(|source| ConfigError::SerializeToml { source })?;

    fs::write(path, content).map_err(|source| ConfigError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

fn apply_patch(table: &mut Table, key: &str, value: Value) {
    let segments: Vec<_> = key.split('.').collect();
    apply_patch_segments(table, &segments, value);
}

fn apply_patch_segments(table: &mut Table, segments: &[&str], value: Value) {
    match segments {
        [] => {}
        [segment] => {
            table.insert((*segment).to_owned(), value);
        }
        [segment, rest @ ..] => {
            let entry = table
                .entry((*segment).to_owned())
                .or_insert_with(|| Value::Table(Table::new()));

            if !entry.is_table() {
                *entry = Value::Table(Table::new());
            }

            if let Some(table) = entry.as_table_mut() {
                apply_patch_segments(table, rest, value);
            }
        }
    }
}

fn toml_value_from_json(
    key: &str,
    value: &JsonValue,
) -> Result<Value, ConfigError> {
    match value {
        JsonValue::Null => Err(ConfigError::InvalidPatchValue {
            key: key.to_owned(),
        }),
        JsonValue::Bool(value) => Ok(Value::Boolean(*value)),
        JsonValue::Number(value) => {
            if let Some(value) = value.as_i64() {
                Ok(Value::Integer(value))
            } else if let Some(value) = value.as_f64() {
                Ok(Value::Float(value))
            } else {
                Err(ConfigError::InvalidPatchValue {
                    key: key.to_owned(),
                })
            }
        }
        JsonValue::String(value) => Ok(Value::String(value.clone())),
        JsonValue::Array(values) => values
            .iter()
            .map(|value| toml_value_from_json(key, value))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        JsonValue::Object(values) => {
            let mut table = Table::new();

            for (child_key, child_value) in values {
                table.insert(
                    child_key.clone(),
                    toml_value_from_json(
                        &format!("{key}.{child_key}"),
                        child_value,
                    )?,
                );
            }

            Ok(Value::Table(table))
        }
    }
}
