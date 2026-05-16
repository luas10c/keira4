# Configuration

The application loads its configuration from `config.toml` inside Tauri's app config directory.

On Linux, with the current app identifier, the file is typically stored at:

```text
~/.config/com.keira4/config.toml
```

## Behavior

- Missing file: uses defaults
- Missing key: uses default for that key
- Invalid key value: uses default for that key
- Structurally invalid TOML: returns an error

## Supported Schema

```toml
language = "en-US"
theme = "system"

[audio]
enabled = true
volume = 0.8

[editor]
cursorSmoothCaretAnimation = "on"
cursorBlinking = "smooth"
fontFamily = 'ui-monospace, "SFMono-Regular", Consolas, "Liberation Mono", monospace'
lineHeight = 1.8
renderLineHighlight = "gutter"
smoothScrolling = true
tabSize = 2

[update]
mode = "default"
showReleaseNotes = true

[window]
titleBarStyle = "custom"
dialogStyle = "custom"
```

## Validation Rules

- `audio.volume`: number between `0` and `1`, accepts integer and float
- `editor.lineHeight`: number greater than `0`, accepts integer and float
- `editor.tabSize`: integer greater than `0`
- `update.mode`: `"default" | "manual" | "none" | "start"`
- `editor.cursorSmoothCaretAnimation`: `"on" | "off" | "explicit"`
- `editor.cursorBlinking`: `"blink" | "smooth" | "phase" | "expand" | "solid"`
- `editor.renderLineHighlight`: `"none" | "gutter" | "line" | "all"`

## Backend Commands

The Tauri backend exposes these commands:

- `load_config`
- `patch_config`
- `set_config_value`

### `load_config`

Returns the normalized configuration with defaults already applied.

```ts
await invoke('load_config')
```

### `set_config_value`

Updates a single config key using dotted path notation.

```ts
await invoke('set_config_value', {
  key: 'editor.tabSize',
  value: 4
})
```

### `patch_config`

Updates multiple config keys in a single operation.

```ts
await invoke('patch_config', {
  patches: [
    { key: 'theme', value: 'dark' },
    { key: 'window.dialogStyle', value: 'native' }
  ]
})
```
