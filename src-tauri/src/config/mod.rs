mod parser;
mod patch;

use std::{path::PathBuf, sync::Mutex};

use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tauri::{Manager, Runtime};

const APP_CONFIG_FILE: &str = "config.toml";

pub use parser::load_from_path;
pub use patch::patch_from_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub language: String,
    pub theme: String,
    pub audio: AudioConfig,
    pub editor: EditorConfig,
    pub update: UpdateConfig,
    pub window: WindowConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigPatch {
    pub key: String,
    pub value: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioConfig {
    pub enabled: bool,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EditorConfig {
    pub cursor_smooth_caret_animation: String,
    pub cursor_blinking: String,
    pub font_family: String,
    pub line_height: f64,
    pub render_line_highlight: String,
    pub smooth_scrolling: bool,
    pub tab_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UpdateConfig {
    pub mode: String,
    pub show_release_notes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowConfig {
    pub title_bar_style: String,
    pub dialog_style: String,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title_bar_style: "custom".into(),
            dialog_style: "custom".into(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 0.8,
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            cursor_smooth_caret_animation: "on".into(),
            cursor_blinking: "smooth".into(),
            font_family: r#"ui-monospace, "SFMono-Regular", Consolas, "Liberation Mono", monospace"#.into(),
            line_height: 1.8,
            render_line_highlight: "gutter".into(),
            smooth_scrolling: true,
            tab_size: 2,
        }
    }
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            mode: "default".into(),
            show_release_notes: true,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "en-US".into(),
            theme: "system".into(),
            audio: AudioConfig::default(),
            editor: EditorConfig::default(),
            update: UpdateConfig::default(),
            window: WindowConfig::default(),
        }
    }
}

pub struct AppConfigState(pub Mutex<AppConfig>);

pub fn config_path<R: Runtime, M: Manager<R>>(
    manager: &M,
) -> Result<PathBuf, ConfigError> {
    manager
        .path()
        .app_config_dir()
        .map(|dir| dir.join(APP_CONFIG_FILE))
        .map_err(|source| ConfigError::ResolveConfigDir { source })
}

#[cfg(test)]
mod tests {
    use super::{load_from_path, patch_from_path, AppConfig, ConfigPatch};
    use serde_json::json;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn unique_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("keira4-config-test-{nanos}"))
    }

    #[test]
    fn returns_defaults_when_file_does_not_exist() {
        let path = unique_test_dir().join("config.toml");

        let config = load_from_path(&path)
            .expect("missing config file should use defaults");

        assert_eq!(config.language, "en-US");
        assert_eq!(config.theme, "system");
        assert!(config.audio.enabled);
        assert_eq!(config.audio.volume, 0.8);
        assert_eq!(config.editor.cursor_smooth_caret_animation, "on");
        assert_eq!(config.editor.cursor_blinking, "smooth");
        assert_eq!(
            config.editor.font_family,
            AppConfig::default().editor.font_family
        );
        assert_eq!(config.editor.line_height, 1.8);
        assert_eq!(config.editor.render_line_highlight, "gutter");
        assert!(config.editor.smooth_scrolling);
        assert_eq!(config.editor.tab_size, 2);
        assert_eq!(config.update.mode, "default");
        assert!(config.update.show_release_notes);
        assert_eq!(config.window.title_bar_style, "custom");
        assert_eq!(config.window.dialog_style, "custom");
    }

    #[test]
    fn keeps_defaults_for_missing_keys() {
        let dir = unique_test_dir();
        let path = dir.join("config.toml");
        fs::create_dir_all(&dir)
            .expect("test config directory should be created");
        fs::write(
            &path,
            "theme = \"dark\"\n[audio]\nvolume = 0.9\n[editor]\ncursorSmoothCaretAnimation = \"explicit\"\ncursorBlinking = \"phase\"\nlineHeight = 2.2\nrenderLineHighlight = \"line\"\nsmoothScrolling = false\ntabSize = 4\n[update]\nmode = \"manual\"\n[window]\ndialogStyle = \"native\"\n",
        )
        .expect("test config file should be written");

        let config =
            load_from_path(&path).expect("partial config should deserialize");

        assert_eq!(config.language, AppConfig::default().language);
        assert_eq!(config.theme, "dark");
        assert!(config.audio.enabled);
        assert_eq!(config.audio.volume, 0.9);
        assert_eq!(config.editor.cursor_smooth_caret_animation, "explicit");
        assert_eq!(config.editor.cursor_blinking, "phase");
        assert_eq!(
            config.editor.font_family,
            AppConfig::default().editor.font_family
        );
        assert_eq!(config.editor.line_height, 2.2);
        assert_eq!(config.editor.render_line_highlight, "line");
        assert!(!config.editor.smooth_scrolling);
        assert_eq!(config.editor.tab_size, 4);
        assert_eq!(config.update.mode, "manual");
        assert!(config.update.show_release_notes);
        assert_eq!(config.window.title_bar_style, "custom");
        assert_eq!(config.window.dialog_style, "native");

        fs::remove_dir_all(&dir)
            .expect("test config directory should be removed");
    }

    #[test]
    fn keeps_defaults_for_invalid_values() {
        let dir = unique_test_dir();
        let path = dir.join("config.toml");
        fs::create_dir_all(&dir)
            .expect("test config directory should be created");
        fs::write(
            &path,
            "language = 10\ntheme = \"dark\"\n[audio]\nenabled = \"yes\"\nvolume = 1.2\n[editor]\ncursorSmoothCaretAnimation = \"fast\"\ncursorBlinking = 99\nfontFamily = 99\nlineHeight = 0\nrenderLineHighlight = \"column\"\nsmoothScrolling = \"yes\"\ntabSize = 0\n[update]\nmode = \"weekly\"\nshowReleaseNotes = \"yes\"\n[window]\ntitleBarStyle = false\ndialogStyle = \"native\"\n",
        )
        .expect("test config file should be written");

        let config = load_from_path(&path)
            .expect("invalid field values should use defaults");

        assert_eq!(config.language, "en-US");
        assert_eq!(config.theme, "dark");
        assert!(config.audio.enabled);
        assert_eq!(config.audio.volume, 0.8);
        assert_eq!(config.editor.cursor_smooth_caret_animation, "on");
        assert_eq!(config.editor.cursor_blinking, "smooth");
        assert_eq!(
            config.editor.font_family,
            AppConfig::default().editor.font_family
        );
        assert_eq!(config.editor.line_height, 1.8);
        assert_eq!(config.editor.render_line_highlight, "gutter");
        assert!(config.editor.smooth_scrolling);
        assert_eq!(config.editor.tab_size, 2);
        assert_eq!(config.update.mode, "default");
        assert!(config.update.show_release_notes);
        assert_eq!(config.window.title_bar_style, "custom");
        assert_eq!(config.window.dialog_style, "native");

        fs::remove_dir_all(&dir)
            .expect("test config directory should be removed");
    }

    #[test]
    fn accepts_integer_audio_volume() {
        let dir = unique_test_dir();
        let path = dir.join("config.toml");
        fs::create_dir_all(&dir)
            .expect("test config directory should be created");
        fs::write(&path, "[audio]\nvolume = 1\n")
            .expect("test config file should be written");

        let config =
            load_from_path(&path).expect("integer volume should deserialize");

        assert_eq!(config.audio.volume, 1.0);

        fs::remove_dir_all(&dir)
            .expect("test config directory should be removed");
    }

    #[test]
    fn accepts_integer_editor_line_height() {
        let dir = unique_test_dir();
        let path = dir.join("config.toml");
        fs::create_dir_all(&dir)
            .expect("test config directory should be created");
        fs::write(&path, "[editor]\nlineHeight = 1\n")
            .expect("test config file should be written");

        let config = load_from_path(&path)
            .expect("integer line height should deserialize");

        assert_eq!(config.editor.line_height, 1.0);

        fs::remove_dir_all(&dir)
            .expect("test config directory should be removed");
    }

    #[test]
    fn reads_theme_from_window_section() {
        let dir = unique_test_dir();
        let path = dir.join("config.toml");
        fs::create_dir_all(&dir)
            .expect("test config directory should be created");
        fs::write(&path, "[window]\ntheme = \"minimal\"\n")
            .expect("test config file should be written");

        let config =
            load_from_path(&path).expect("window theme should deserialize");

        assert_eq!(config.theme, "minimal");

        fs::remove_dir_all(&dir)
            .expect("test config directory should be removed");
    }

    #[test]
    fn patches_single_config_key() {
        let dir = unique_test_dir();
        let path = dir.join("config.toml");

        let config = patch_from_path(
            &path,
            &[ConfigPatch {
                key: "theme".into(),
                value: json!("dark"),
            }],
        )
        .expect("single config patch should be written");

        assert_eq!(config.theme, "dark");
        assert_eq!(
            load_from_path(&path)
                .expect("patched config should load")
                .theme,
            "dark"
        );

        fs::remove_dir_all(&dir)
            .expect("test config directory should be removed");
    }

    #[test]
    fn patches_multiple_config_keys() {
        let dir = unique_test_dir();
        let path = dir.join("config.toml");
        fs::create_dir_all(&dir)
            .expect("test config directory should be created");
        fs::write(&path, "theme = \"system\"\n")
            .expect("test config file should be written");

        let config = patch_from_path(
            &path,
            &[
                ConfigPatch {
                    key: "editor.tabSize".into(),
                    value: json!(4),
                },
                ConfigPatch {
                    key: "window.dialogStyle".into(),
                    value: json!("native"),
                },
            ],
        )
        .expect("multiple config patches should be written");

        assert_eq!(config.editor.tab_size, 4);
        assert_eq!(config.window.dialog_style, "native");
        assert_eq!(config.theme, "system");

        let persisted =
            load_from_path(&path).expect("patched config should load");
        assert_eq!(persisted.editor.tab_size, 4);
        assert_eq!(persisted.window.dialog_style, "native");
        assert_eq!(persisted.theme, "system");

        fs::remove_dir_all(&dir)
            .expect("test config directory should be removed");
    }

    #[test]
    #[cfg(unix)]
    fn patches_config_when_target_file_is_read_only() {
        let dir = unique_test_dir();
        let path = dir.join("config.toml");
        fs::create_dir_all(&dir)
            .expect("test config directory should be created");
        fs::write(&path, "theme = \"minimal\"\n")
            .expect("test config file should be written");

        let mut permissions = fs::metadata(&path)
            .expect("test config metadata should exist")
            .permissions();
        permissions.set_mode(0o444);
        fs::set_permissions(&path, permissions)
            .expect("test config file should be set read-only");

        let updated = patch_from_path(
            &path,
            &[ConfigPatch {
                key: "theme".into(),
                value: json!("midnight"),
            }],
        )
        .expect("patching should replace read-only file atomically");

        assert_eq!(updated.theme, "midnight");
        let persisted = load_from_path(&path)
            .expect("updated config should be readable");
        assert_eq!(persisted.theme, "midnight");

        fs::remove_dir_all(&dir)
            .expect("test config directory should be removed");
    }

    #[test]
    fn returns_error_for_invalid_toml() {
        let dir = unique_test_dir();
        let path = dir.join("config.toml");
        fs::create_dir_all(&dir)
            .expect("test config directory should be created");
        fs::write(&path, "theme = [\n")
            .expect("invalid config file should be written");

        let error =
            load_from_path(&path).expect_err("invalid TOML should fail");

        assert!(error.to_string().contains("invalid TOML"));

        fs::remove_dir_all(&dir)
            .expect("test config directory should be removed");
    }
}
