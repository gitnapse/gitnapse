use serde::Deserialize;
use std::path::PathBuf;

use crate::config::{config_dir, strip_jsonc_comments};

/// A color value parsed from JSON: either an RGB array `[r, g, b]` or a hex
/// string `"#RRGGBB"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorValue {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorValue {
    pub fn as_rgb(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim();
        let s = s.strip_prefix('#').unwrap_or(s);
        if s.len() != 6 {
            return None;
        }
        let hex = u32::from_str_radix(s, 16).ok()?;
        Some(Self {
            r: ((hex >> 16) & 0xff) as u8,
            g: ((hex >> 8) & 0xff) as u8,
            b: (hex & 0xff) as u8,
        })
    }
}

impl<'de> Deserialize<'de> for ColorValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Array([u8; 3]),
            Hex(String),
        }
        match Repr::deserialize(deserializer)? {
            Repr::Array(rgb) => Ok(Self {
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
            }),
            Repr::Hex(s) => Self::from_hex(&s)
                .ok_or_else(|| serde::de::Error::custom(format!("invalid color: {s}"))),
        }
    }
}

/// Resolved semantic colors for the active theme.
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    /// Base background.
    pub background: [u8; 3],
    /// Base foreground (font color).
    pub foreground: [u8; 3],
    /// Primary accent (selection / focus).
    pub accent: [u8; 3],
    pub accent2: [u8; 3],
    pub accent3: [u8; 3],
    /// Optional explicit font color used on accent backgrounds.
    pub selection_fg: Option<[u8; 3]>,
}

/// Theme configuration. Supports both the legacy full `palette` and a
/// simplified set of colors (`background`, `foreground`, `accent*`).
#[derive(Debug, Clone, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_theme_name")]
    pub theme_name: String,
    #[serde(default)]
    pub background: Option<ColorValue>,
    #[serde(default)]
    pub foreground: Option<ColorValue>,
    #[serde(default)]
    pub accent: Option<ColorValue>,
    #[serde(default)]
    pub accent2: Option<ColorValue>,
    #[serde(default)]
    pub accent3: Option<ColorValue>,
    #[serde(default)]
    pub selection_fg: Option<ColorValue>,
    #[serde(default)]
    pub palette: Option<Vec<[u8; 3]>>,
}

fn default_theme_name() -> String {
    "X".to_string()
}

const DEFAULT_BG: ColorValue = ColorValue {
    r: 0x05,
    g: 0x05,
    b: 0x05,
};
const DEFAULT_FG: ColorValue = ColorValue {
    r: 0xf7,
    g: 0xf1,
    b: 0xff,
};
const DEFAULT_ACCENT: ColorValue = ColorValue {
    r: 0xfc,
    g: 0x61,
    b: 0x8d,
};
const DEFAULT_ACCENT2: ColorValue = ColorValue {
    r: 0x7b,
    g: 0xd8,
    b: 0x8f,
};
const DEFAULT_ACCENT3: ColorValue = ColorValue {
    r: 0x5a,
    g: 0xd4,
    b: 0xe6,
};

/// The default theme is the "X" palette expressed in the simplified format.
impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            theme_name: "X".to_string(),
            background: Some(DEFAULT_BG),
            foreground: Some(DEFAULT_FG),
            accent: Some(DEFAULT_ACCENT),
            accent2: Some(DEFAULT_ACCENT2),
            accent3: Some(DEFAULT_ACCENT3),
            selection_fg: Some(ColorValue {
                r: 0x00,
                g: 0x00,
                b: 0x00,
            }),
            palette: None,
        }
    }
}

impl ThemeConfig {
    /// Resolve the semantic colors, falling back to defaults for anything unset.
    pub fn effective_colors(&self) -> ThemeColors {
        let bg = self.background.unwrap_or(DEFAULT_BG).as_rgb();
        let fg = self.foreground.unwrap_or(DEFAULT_FG).as_rgb();
        let accent = self.accent.unwrap_or(DEFAULT_ACCENT).as_rgb();
        ThemeColors {
            background: bg,
            foreground: fg,
            accent,
            accent2: self.accent2.unwrap_or(DEFAULT_ACCENT2).as_rgb(),
            accent3: self.accent3.unwrap_or(DEFAULT_ACCENT3).as_rgb(),
            selection_fg: self.selection_fg.map(|c| c.as_rgb()),
        }
    }

    /// The effective 16-entry palette: the legacy `palette` when present,
    /// otherwise derived from the simplified colors.
    pub fn effective_palette(&self) -> Vec<[u8; 3]> {
        if let Some(palette) = &self.palette
            && !palette.is_empty()
        {
            return palette.clone();
        }
        let c = self.effective_colors();
        let accent = c.accent;
        let accent2 = c.accent2;
        let accent3 = c.accent3;
        let bg = c.background;
        let fg = c.foreground;
        let dim = lighten(bg, 0.35);
        let first = [bg, accent, accent2, accent3, accent, accent2, accent3, fg];
        let mut palette = first.to_vec();
        palette.push(dim);
        // Repeat the 7 accent/foreground entries to complete 16 (legacy layout).
        palette.extend_from_slice(&first[1..]);
        palette
    }

    /// Load the active theme: the named theme from `theme.jsonc` (or default),
    /// with any inline color fields in `theme.jsonc` overriding it.
    pub fn load_or_default() -> Self {
        let dir = match config_dir() {
            Ok(d) => d,
            Err(_) => return Self::default(),
        };

        // Auto-install/update built-in themes
        let themes_dir = dir.join("themes");
        let _ = std::fs::create_dir_all(&themes_dir);

        let builtin_sources: Vec<PathBuf> = {
            let mut sources = Vec::new();
            if let Ok(exe_dir) = std::env::current_exe()
                && let Some(exe_parent) = exe_dir.parent()
            {
                let p = exe_parent.join("../themes");
                if p.exists() {
                    sources.push(p);
                }
            }
            if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
                let p = std::path::PathBuf::from(manifest_dir).join("themes");
                if p.exists() {
                    sources.push(p);
                }
            }
            sources
        };

        for src_dir in &builtin_sources {
            if let Ok(entries) = std::fs::read_dir(src_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "jsonc")
                        && let Some(name) = path.file_name()
                    {
                        let dest = themes_dir.join(name);
                        let should_copy = match std::fs::metadata(&dest) {
                            Ok(dest_meta) => match std::fs::metadata(&path) {
                                Ok(src_meta) => {
                                    src_meta.modified().ok() > dest_meta.modified().ok()
                                }
                                Err(_) => false,
                            },
                            Err(_) => true,
                        };
                        if should_copy {
                            let _ = std::fs::copy(&path, &dest);
                        }
                    }
                }
            }
        }

        // Read theme.jsonc (the recommended file for customization).
        let user = read_theme_jsonc(&dir.join("theme.jsonc"));

        // Load the selected named theme, then overlay the user's overrides.
        let mut base = Self::load_named(&user.theme_name);
        base.theme_name = user.theme_name;
        if let Some(v) = user.background {
            base.background = Some(v);
        }
        if let Some(v) = user.foreground {
            base.foreground = Some(v);
        }
        if let Some(v) = user.accent {
            base.accent = Some(v);
        }
        if let Some(v) = user.accent2 {
            base.accent2 = Some(v);
        }
        if let Some(v) = user.accent3 {
            base.accent3 = Some(v);
        }
        if let Some(v) = user.selection_fg {
            base.selection_fg = Some(v);
        }
        if let Some(v) = user.palette {
            base.palette = Some(v);
        }
        base
    }

    /// Load a named theme file (config dir first, then built-in/themes dir).
    pub fn load_named(name: &str) -> Self {
        if let Some(theme_path) = Self::theme_file_path(name)
            && theme_path.exists()
            && let Ok(raw) = std::fs::read_to_string(&theme_path)
        {
            let cleaned = strip_jsonc_comments(&raw);
            if let Ok(cfg) = serde_json::from_str::<Self>(&cleaned) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn theme_file_path(name: &str) -> Option<PathBuf> {
        if let Ok(dir) = config_dir() {
            let path = dir.join("themes").join(format!("{name}.jsonc"));
            if path.exists() {
                return Some(path);
            }
        }

        if let Ok(exe_dir) = std::env::current_exe()
            && let Some(exe_parent) = exe_dir.parent()
        {
            let path = exe_parent.join("../themes").join(format!("{name}.jsonc"));
            if path.exists() {
                return Some(path);
            }
        }

        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let path = PathBuf::from(manifest_dir)
                .join("themes")
                .join(format!("{name}.jsonc"));
            if path.exists() {
                return Some(path);
            }
        }

        None
    }
}

fn read_theme_jsonc(path: &PathBuf) -> ThemeConfig {
    if path.exists()
        && let Ok(raw) = std::fs::read_to_string(path)
    {
        let cleaned = strip_jsonc_comments(&raw);
        if let Ok(cfg) = serde_json::from_str::<ThemeConfig>(&cleaned) {
            return cfg;
        }
    }
    ThemeConfig::default()
}

/// Blend `rgb` toward white by `amount` (0..1). Used for the "dim" palette slot.
fn lighten(rgb: [u8; 3], amount: f64) -> [u8; 3] {
    let blend = |c: u8| (f64::from(c) + (255.0 - f64::from(c)) * amount).round() as u8;
    [blend(rgb[0]), blend(rgb[1]), blend(rgb[2])]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_color() {
        let c = ColorValue::from_hex("#363537").unwrap();
        assert_eq!(c.as_rgb(), [0x36, 0x35, 0x37]);
    }

    #[test]
    fn rejects_bad_hex() {
        assert!(ColorValue::from_hex("#12345").is_none());
        assert!(ColorValue::from_hex("zzzzzz").is_none());
    }

    #[test]
    fn default_derives_full_palette() {
        let t = ThemeConfig::default();
        assert_eq!(t.effective_palette().len(), 16);
        assert_eq!(t.effective_palette()[0], [0x05, 0x05, 0x05]);
        assert_eq!(t.effective_palette()[7], [0xf7, 0xf1, 0xff]);
    }

    #[test]
    fn legacy_palette_wins_over_colors() {
        let t = ThemeConfig {
            palette: Some(vec![[1, 2, 3]]),
            ..ThemeConfig::default()
        };
        let p = t.effective_palette();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0], [1, 2, 3]);
    }

    #[test]
    fn custom_hex_colors_derive_palette() {
        let t = serde_json::from_str::<ThemeConfig>(
            r##"{"background":"#000000","foreground":"#ffffff","accent":"#ff0000"}"##,
        )
        .unwrap();
        let p = t.effective_palette();
        assert_eq!(p[0], [0, 0, 0]);
        assert_eq!(p[1], [0xff, 0, 0]);
        assert_eq!(p[7], [0xff, 0xff, 0xff]);
        let c = t.effective_colors();
        assert_eq!(c.background, [0, 0, 0]);
        assert_eq!(c.foreground, [0xff, 0xff, 0xff]);
    }
}
