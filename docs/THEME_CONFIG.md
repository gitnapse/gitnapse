# Theme Configuration

GitNapse supports custom color theming through an optional `theme.jsonc` file placed in the configuration directory. This is the **recommended file** to choose colors and the font color.

## Location

The configuration directory is platform-dependent:

| Platform | Path |
|---|---|
| Linux | `~/.config/GitNapse/` |
| macOS | `~/Library/Application Support/com.GitNapse.GitNapse/` |
| Windows | `C:\Users\<user>\AppData\Roaming\GitNapse\GitNapse\config\` |

Place the file at `theme.jsonc` inside that directory. If the file does not exist, GitNapse uses the built-in default theme (`X`).

## Simplified format

Pick a named theme and/or set your own colors. Colors accept hex strings (`"#RRGGBB"`) or RGB arrays (`[r, g, b]`).

```jsonc
{
    // Pick a built-in theme (optional; the base for your overrides).
    "theme_name": "X",

    // Base colors — the two you'll want most:
    "background": "#1e1e2e",   // background color
    "foreground": "#cdd6f4",   // font color

    // Accents used for selection, focus, and pane borders (optional):
    "accent": "#89b4fa",       // primary accent
    "accent2": "#a6e3a1",      // secondary accent
    "accent3": "#f38f48",      // tertiary accent
    "selection_fg": "#11111b"  // font color used on accent backgrounds (optional)
}
```

The `accent*` colors are cycled for selection highlighting and pane borders. `foreground` is applied to the font of the main UI (search bar, status bar, lists, preview). `selection_fg` controls the text on selected rows when you want an explicit contrast color; otherwise GitNapse picks black/white automatically based on luminance.

You only need to provide the fields you want — everything unset falls back to the selected theme's values.

## Example: full custom theme

```jsonc
{
    "background": "#000000",
    "foreground": "#ffffff",
    "accent": "#ff5555",
    "accent2": "#55ff55",
    "accent3": "#5555ff"
}
```

## Named theme files

Built-in themes live as `themes/*.jsonc` (config dir, then the app's `themes/` directory). They use the same format:

```jsonc
{
    "theme_name": "X",
    "background": "#363537",
    "foreground": "#f7f1ff",
    "accent": "#fc618d",
    "accent2": "#7bd88f",
    "accent3": "#5ad4e6"
}
```

## Legacy palette

The old 16-color `palette` array is still supported. When present, it takes precedence over the simplified colors (no palette is derived):

```jsonc
{
    "palette": [
        [0x36, 0x35, 0x37],   // index 0  - dark background
        [0xfc, 0x61, 0x8d],   // index 1  - pink
        // ... up to 16 entries
    ]
}
```

The `palette` is indexed cyclically for selection highlighting. If you provide fewer than 16 entries they repeat.

## Error handling

If `theme.jsonc` is absent or contains invalid JSON, GitNapse silently falls back to the default theme. No error is shown to the user.
