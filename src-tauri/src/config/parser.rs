use std::{fs, path::Path};

use toml::{Table, Value};

use crate::error::ConfigError;

use super::{AppConfig, AudioConfig, EditorConfig, UpdateConfig, WindowConfig};

impl AppConfig {
    pub(super) fn from_table(table: &Table) -> Self {
        let defaults = Self::default();

        Self {
            language: string_value(table, "language", &defaults.language),
            theme: string_value(table, "theme", &defaults.theme),
            audio: AudioConfig::from_table(
                table.get("audio").and_then(Value::as_table),
            ),
            editor: EditorConfig::from_table(
                table.get("editor").and_then(Value::as_table),
            ),
            update: UpdateConfig::from_table(
                table.get("update").and_then(Value::as_table),
            ),
            window: WindowConfig::from_table(
                table.get("window").and_then(Value::as_table),
            ),
        }
    }
}

impl AudioConfig {
    fn from_table(table: Option<&Table>) -> Self {
        let defaults = Self::default();

        match table {
            Some(table) => Self {
                enabled: bool_value(table, "enabled", defaults.enabled),
                volume: volume_value(table, "volume", defaults.volume),
            },
            None => defaults,
        }
    }
}

impl WindowConfig {
    fn from_table(table: Option<&Table>) -> Self {
        let defaults = Self::default();

        match table {
            Some(table) => Self {
                title_bar_style: string_value(
                    table,
                    "titleBarStyle",
                    &defaults.title_bar_style,
                ),
                dialog_style: string_value(
                    table,
                    "dialogStyle",
                    &defaults.dialog_style,
                ),
            },
            None => defaults,
        }
    }
}

impl EditorConfig {
    fn from_table(table: Option<&Table>) -> Self {
        let defaults = Self::default();

        match table {
            Some(table) => Self {
                cursor_smooth_caret_animation:
                    editor_cursor_smooth_caret_animation_value(
                        table,
                        "cursorSmoothCaretAnimation",
                        &defaults.cursor_smooth_caret_animation,
                    ),
                cursor_blinking: editor_cursor_blinking_value(
                    table,
                    "cursorBlinking",
                    &defaults.cursor_blinking,
                ),
                font_family: string_value(
                    table,
                    "fontFamily",
                    &defaults.font_family,
                ),
                line_height: positive_numeric_value(
                    table,
                    "lineHeight",
                    defaults.line_height,
                ),
                render_line_highlight: editor_render_line_highlight_value(
                    table,
                    "renderLineHighlight",
                    &defaults.render_line_highlight,
                ),
                smooth_scrolling: bool_value(
                    table,
                    "smoothScrolling",
                    defaults.smooth_scrolling,
                ),
                tab_size: positive_integer_value(
                    table,
                    "tabSize",
                    defaults.tab_size,
                ),
            },
            None => defaults,
        }
    }
}

impl UpdateConfig {
    fn from_table(table: Option<&Table>) -> Self {
        let defaults = Self::default();

        match table {
            Some(table) => Self {
                mode: update_mode_value(table, "mode", &defaults.mode),
                show_release_notes: bool_value(
                    table,
                    "showReleaseNotes",
                    defaults.show_release_notes,
                ),
            },
            None => defaults,
        }
    }
}

pub fn load_from_path(path: &Path) -> Result<AppConfig, ConfigError> {
    if !path.exists() {
        return Ok(AppConfig::default());
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

    Ok(match value.as_table() {
        Some(table) => AppConfig::from_table(table),
        None => AppConfig::default(),
    })
}

fn string_value(table: &Table, key: &str, default: &str) -> String {
    table
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_owned()
}

fn bool_value(table: &Table, key: &str, default: bool) -> bool {
    table.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn positive_integer_value(table: &Table, key: &str, default: i64) -> i64 {
    match table.get(key).and_then(Value::as_integer) {
        Some(value) if value > 0 => value,
        _ => default,
    }
}

fn positive_numeric_value(table: &Table, key: &str, default: f64) -> f64 {
    match table.get(key).and_then(numeric_value) {
        Some(value) if value > 0.0 => value,
        _ => default,
    }
}

fn editor_cursor_smooth_caret_animation_value(
    table: &Table,
    key: &str,
    default: &str,
) -> String {
    enum_string_value(table, key, &["on", "off", "explicit"], default)
}

fn editor_cursor_blinking_value(
    table: &Table,
    key: &str,
    default: &str,
) -> String {
    enum_string_value(
        table,
        key,
        &["blink", "smooth", "phase", "expand", "solid"],
        default,
    )
}

fn editor_render_line_highlight_value(
    table: &Table,
    key: &str,
    default: &str,
) -> String {
    enum_string_value(table, key, &["none", "gutter", "line", "all"], default)
}

fn enum_string_value(
    table: &Table,
    key: &str,
    allowed: &[&str],
    default: &str,
) -> String {
    match table.get(key).and_then(Value::as_str) {
        Some(value) if allowed.contains(&value) => value.to_owned(),
        _ => default.to_owned(),
    }
}

fn update_mode_value(table: &Table, key: &str, default: &str) -> String {
    match table.get(key).and_then(Value::as_str) {
        Some(value @ ("default" | "manual" | "none" | "start")) => {
            value.to_owned()
        }
        _ => default.to_owned(),
    }
}

fn volume_value(table: &Table, key: &str, default: f64) -> f64 {
    match table.get(key).and_then(numeric_value) {
        Some(volume) if (0.0..=1.0).contains(&volume) => volume,
        _ => default,
    }
}

fn numeric_value(value: &Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|number| number as f64))
}
