use crate::config::{
    KeybindingsConfig, ThemeColors, ThemeConfig, config_dir, strip_jsonc_comments,
};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::sync::{LazyLock, RwLock};

/// Embedded theme data: (name, JSONC content).
/// Bundled at compile time so themes are available after `cargo install`.
const EMBEDDED_THEMES: &[(&str, &str)] = &[
    ("Berlin", include_str!("../../themes/Berlin.jsonc")),
    ("Bogota", include_str!("../../themes/Bogota.jsonc")),
    ("Helsinki", include_str!("../../themes/Helsinki.jsonc")),
    ("Lahabana", include_str!("../../themes/Lahabana.jsonc")),
    ("London", include_str!("../../themes/London.jsonc")),
    ("Madrid", include_str!("../../themes/Madrid.jsonc")),
    ("Miami", include_str!("../../themes/Miami.jsonc")),
    ("Oslo", include_str!("../../themes/Oslo.jsonc")),
    ("Paris", include_str!("../../themes/Paris.jsonc")),
    ("Praha", include_str!("../../themes/Praha.jsonc")),
    ("Tokio", include_str!("../../themes/Tokio.jsonc")),
    ("X", include_str!("../../themes/X.jsonc")),
];

const DEFAULT_PALETTE: [[u8; 3]; 16] = [
    [0x36, 0x35, 0x37],
    [0xfc, 0x61, 0x8d],
    [0x7b, 0xd8, 0x8f],
    [0xfc, 0xe5, 0x66],
    [0xfd, 0x93, 0x53],
    [0x94, 0x8a, 0xe3],
    [0x5a, 0xd4, 0xe6],
    [0xf7, 0xf1, 0xff],
    [0x69, 0x67, 0x6c],
    [0xfc, 0x61, 0x8d],
    [0x7b, 0xd8, 0x8f],
    [0xfc, 0xe5, 0x66],
    [0xfd, 0x93, 0x53],
    [0x94, 0x8a, 0xe3],
    [0x5a, 0xd4, 0xe6],
    [0xf7, 0xf1, 0xff],
];

static PALETTE: LazyLock<RwLock<Vec<[u8; 3]>>> =
    LazyLock::new(|| RwLock::new(DEFAULT_PALETTE.to_vec()));

static SEMANTIC: LazyLock<RwLock<ThemeColors>> =
    LazyLock::new(|| RwLock::new(ThemeConfig::default().effective_colors()));

pub fn init_theme(config: &ThemeConfig) {
    if let Ok(mut p) = PALETTE.write() {
        *p = config.effective_palette();
    }
    if let Ok(mut s) = SEMANTIC.write() {
        *s = config.effective_colors();
    }
}

/// Base background color.
pub fn background() -> Color {
    semantic_rgb(|s| s.background)
}

/// Base foreground (font color).
pub fn foreground() -> Color {
    semantic_rgb(|s| s.foreground)
}

fn semantic_rgb(slot: fn(&ThemeColors) -> [u8; 3]) -> Color {
    let rgb = match SEMANTIC.read() {
        Ok(guard) => slot(&guard),
        Err(_) => [0xfc, 0x61, 0x8d],
    };
    Color::Rgb(rgb[0], rgb[1], rgb[2])
}

/// Primary accent.
pub fn accent() -> Color {
    semantic_rgb(|s| s.accent)
}

pub fn accent2() -> Color {
    semantic_rgb(|s| s.accent2)
}

pub fn accent3() -> Color {
    semantic_rgb(|s| s.accent3)
}

/// Base text style honoring the theme's background and font color.
pub fn text_style() -> Style {
    Style::default().fg(foreground()).bg(background())
}

pub fn load_theme_by_name(name: &str) -> ThemeConfig {
    // Try external themes directory first (allows user overrides).
    if let Ok(dir) = config_dir() {
        let path = dir.join("themes").join(format!("{name}.jsonc"));
        if path.exists()
            && let Ok(raw) = std::fs::read_to_string(&path)
        {
            let cleaned = strip_jsonc_comments(&raw);
            if let Ok(cfg) = serde_json::from_str(&cleaned) {
                return cfg;
            }
        }
    }
    // Fall back to embedded themes.
    for (embedded_name, content) in EMBEDDED_THEMES {
        if *embedded_name == name {
            let cleaned = strip_jsonc_comments(content);
            if let Ok(cfg) = serde_json::from_str(&cleaned) {
                return cfg;
            }
        }
    }
    ThemeConfig::default()
}

fn collect_themes_from(dir: &std::path::Path, names: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonc")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                let s = stem.to_string();
                if !names.contains(&s) {
                    names.push(s);
                }
            }
        }
    }
}

pub fn list_available_themes() -> Vec<String> {
    let mut names = Vec::new();

    // External themes take precedence (user overrides).
    if let Ok(dir) = config_dir() {
        collect_themes_from(&dir.join("themes"), &mut names);
    }

    // Always include embedded themes.
    for (name, _) in EMBEDDED_THEMES {
        let s = name.to_string();
        if !names.contains(&s) {
            names.push(s);
        }
    }

    names.sort();
    names
}

fn palette() -> Vec<[u8; 3]> {
    PALETTE
        .read()
        .ok()
        .map(|g| g.clone())
        .unwrap_or(DEFAULT_PALETTE.to_vec())
}

#[cfg(test)]
pub fn palette_len() -> usize {
    palette().len()
}

fn palette_rgb(index: usize) -> (u8, u8, u8) {
    let pal = palette();
    let entry = pal[index % pal.len()];
    (entry[0], entry[1], entry[2])
}

fn relative_luminance(rgb: (u8, u8, u8)) -> f64 {
    let linearize = |c: f64| {
        if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (
        linearize(f64::from(rgb.0) / 255.0),
        linearize(f64::from(rgb.1) / 255.0),
        linearize(f64::from(rgb.2) / 255.0),
    );
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast_ratio(a: (u8, u8, u8), b: (u8, u8, u8)) -> f64 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Pick the readable text color (black or white) for a background using the
/// WCAG contrast ratio rather than a fixed luminance threshold.
fn contrast_fg_from_rgb(rgb: (u8, u8, u8)) -> Color {
    let black = contrast_ratio(rgb, (0, 0, 0));
    let white = contrast_ratio(rgb, (255, 255, 255));
    if black >= white {
        Color::Black
    } else {
        Color::White
    }
}

pub fn selection_style(index: usize) -> Style {
    let rgb = palette_rgb(index);
    let fg = SEMANTIC
        .read()
        .ok()
        .and_then(|s| s.selection_fg)
        .map(|rgb| Color::Rgb(rgb[0], rgb[1], rgb[2]))
        .unwrap_or_else(|| contrast_fg_from_rgb(rgb));
    Style::default()
        .bg(Color::Rgb(rgb.0, rgb.1, rgb.2))
        .fg(fg)
        .add_modifier(Modifier::BOLD)
}

fn nav_labels(kb: &KeybindingsConfig) -> Vec<String> {
    fn key_str(keys: &[String]) -> String {
        keys.join("/")
    }
    vec![
        format!(" {} Search ", key_str(std::slice::from_ref(&kb.search))),
        format!(
            " {} Open/Preview ",
            key_str(std::slice::from_ref(&kb.enter))
        ),
        format!(" {}/{} Move ", kb.scroll_up, kb.scroll_down),
        format!(" {} Prev Page ", key_str(&kb.page_left)),
        format!(" {} Next Page ", key_str(&kb.page_right)),
        format!(" {} Repos/Tree/Preview ", kb.focus_next),
        format!(" {}/{} Preview ", kb.page_up, kb.page_down),
        format!(" {}/{} Preview ", kb.home, kb.end),
        format!(" {} Branch ", kb.branch_picker),
        format!(" {} Find File ", kb.file_search),
        format!(" {} Tree View ", kb.tree_view),
        format!(" {} Download File ", kb.download),
        format!(" {} Clone ", kb.clone),
        format!(" {} Token ", kb.token_input),
        format!(" {} OAuth State ", kb.oauth_status),
        format!(" {} Back ", kb.escape),
        " Mouse Click/Scroll ".to_string(),
    ]
}

pub fn nav_hint_lines(kb: &KeybindingsConfig, max_width: usize) -> Vec<Line<'static>> {
    let width = max_width.max(20);
    let labels = nav_labels(kb);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut current_width = 0usize;

    for (index, label) in labels.iter().enumerate() {
        let label_w = label.chars().count();
        if current_width + label_w > width && !current.is_empty() {
            lines.push(Line::from(current));
            current = Vec::new();
            current_width = 0;
        }
        current.push(Span::styled(label.to_string(), selection_style(index)));
        current_width += label_w;
    }

    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    if lines.is_empty() {
        lines.push(Line::from(" Navigation "));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn uses_full_reference_palette() {
        assert_eq!(palette_len(), 16);
    }

    #[test]
    fn picks_white_on_dark_and_black_on_light() {
        assert_eq!(contrast_fg_from_rgb((0x36, 0x35, 0x37)), Color::White);
        assert_eq!(contrast_fg_from_rgb((0xf7, 0xf1, 0xff)), Color::Black);
    }

    #[test]
    fn embedded_themes_have_legible_font_on_background() {
        for (name, _) in EMBEDDED_THEMES {
            let cfg = load_theme_by_name(name);
            let colors = cfg.effective_colors();
            let bg = (
                colors.background[0],
                colors.background[1],
                colors.background[2],
            );
            let fg = (
                colors.foreground[0],
                colors.foreground[1],
                colors.foreground[2],
            );
            let ratio = contrast_ratio(bg, fg);
            assert!(
                ratio >= 4.5,
                "{name}: foreground/background contrast {ratio:.2} < 4.5 (AA)"
            );
        }
    }

    #[test]
    fn init_theme_changes_palette() {
        let default_rgb = palette_rgb(1);
        // Debug: show embedded Berlin content
        let embedded = EMBEDDED_THEMES.iter().find(|(n, _)| *n == "Berlin");
        assert!(embedded.is_some(), "Berlin not in EMBEDDED_THEMES");
        if let Some((_, content)) = embedded {
            assert!(!content.is_empty(), "Berlin embed content is empty");
            eprintln!("Berlin embed content: {content}");
        }
        // Verify load_theme_by_name returns Berlin
        let berlin = load_theme_by_name("Berlin");
        let berlin_palette = berlin.effective_palette();
        assert_eq!(
            berlin_palette.len(),
            16,
            "Berlin palette should have 16 entries"
        );
        assert_eq!(berlin_palette[0], [0, 0, 0], "first entry should be black");
        // Apply Berlin theme
        init_theme(&berlin);
        let berlin_rgb = palette_rgb(1);
        assert_eq!(
            berlin_rgb,
            (153, 153, 153),
            "expected Berlin gray, got {berlin_rgb:?}"
        );
        // Reset to default
        init_theme(&ThemeConfig::default());
        let reset_rgb = palette_rgb(1);
        assert_eq!(reset_rgb, default_rgb, "should restore default");
    }
}
