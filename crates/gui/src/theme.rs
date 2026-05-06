//! DeepSeek GUI theme — Qoder-style compact dark UI.

use egui::{Color32, FontFamily, FontId, FontDefinitions, Style, TextStyle, Visuals};

/// DeepSeek brand colors.
pub struct DeepSeekColors;

impl DeepSeekColors {
    pub const BACKGROUND: Color32 = Color32::from_rgb(13, 17, 23);
    pub const SURFACE: Color32 = Color32::from_rgb(22, 27, 34);
    pub const SURFACE_HOVER: Color32 = Color32::from_rgb(33, 38, 45);
    pub const BORDER: Color32 = Color32::from_rgb(33, 38, 45);
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(201, 209, 217);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(139, 148, 158);
    pub const ACCENT: Color32 = Color32::from_rgb(88, 166, 255);
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(121, 184, 255);
    pub const USER_BUBBLE: Color32 = Color32::from_rgb(31, 111, 235);
    pub const ASSISTANT_BUBBLE: Color32 = Color32::from_rgb(33, 38, 45);
    pub const THINKING: Color32 = Color32::from_rgb(139, 148, 158);
    pub const CODE_BG: Color32 = Color32::from_rgb(22, 27, 34);
    pub const SUCCESS: Color32 = Color32::from_rgb(63, 185, 80);
    pub const WARNING: Color32 = Color32::from_rgb(210, 153, 34);
    pub const ERROR: Color32 = Color32::from_rgb(248, 81, 73);
}

/// Apply the Qoder-style dark theme to the egui context.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(DeepSeekColors::TEXT_PRIMARY);
    visuals.panel_fill = DeepSeekColors::BACKGROUND;
    visuals.window_fill = DeepSeekColors::SURFACE;
    visuals.window_stroke = egui::Stroke::new(1.0, DeepSeekColors::BORDER);
    visuals.widgets.noninteractive.bg_fill = DeepSeekColors::SURFACE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, DeepSeekColors::TEXT_PRIMARY);
    visuals.widgets.inactive.bg_fill = DeepSeekColors::SURFACE_HOVER;
    visuals.widgets.inactive.weak_bg_fill = DeepSeekColors::SURFACE_HOVER;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, DeepSeekColors::TEXT_PRIMARY);
    visuals.widgets.hovered.bg_fill = DeepSeekColors::SURFACE_HOVER;
    visuals.widgets.active.bg_fill = DeepSeekColors::BORDER;
    visuals.widgets.open.bg_fill = DeepSeekColors::SURFACE_HOVER;
    visuals.selection.bg_fill = DeepSeekColors::SURFACE_HOVER;
    visuals.selection.stroke = egui::Stroke::new(1.0, DeepSeekColors::ACCENT);
    visuals.hyperlink_color = DeepSeekColors::ACCENT;
    visuals.faint_bg_color = DeepSeekColors::SURFACE;
    visuals.extreme_bg_color = DeepSeekColors::CODE_BG;
    visuals.code_bg_color = DeepSeekColors::CODE_BG;
    visuals.window_rounding = egui::Rounding::same(6.0);
    visuals.menu_rounding = egui::Rounding::same(4.0);
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;

    let mut style = Style {
        visuals,
        ..Style::default()
    };

    // Compact spacing (Qoder-style)
    style.spacing.item_spacing = egui::vec2(4.0, 2.0);
    style.spacing.button_padding = egui::vec2(6.0, 2.0);
    style.spacing.indent = 16.0;
    style.spacing.icon_width = 12.0;
    style.spacing.icon_spacing = 4.0;

    // Qoder-style font sizes (smaller, tighter)
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(18.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(13.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(13.0, FontFamily::Monospace),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(13.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(11.0, FontFamily::Proportional),
    );

    // Load fonts: Segoe UI (UI), Consolas (code), CJK fallback
    let mut fonts = FontDefinitions::default();
    load_system_fonts(&mut fonts);
    ctx.set_fonts(fonts);

    ctx.set_style(style);
}

/// Load bundled fonts first (Inter, JetBrains Mono), then fall back to system fonts.
fn load_system_fonts(fonts: &mut FontDefinitions) {
    // Try bundled UI font (Inter) first
    let bundled_ui = [
        "crates/gui/assets/fonts/InterVariable.ttf",
        "crates/gui/assets/fonts/Inter-Regular.ttf",
    ];
    for path in &bundled_ui {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "inter".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "inter".to_owned());
            break;
        }
    }

    // Try bundled code font (JetBrains Mono Nerd Font) first
    let bundled_code = [
        "crates/gui/assets/fonts/JetBrainsMonoNerdFont-Regular.ttf",
        "crates/gui/assets/fonts/JetBrainsMono-Regular.ttf",
        "crates/gui/assets/fonts/JetBrainsMono-Variable.ttf",
    ];
    for path in &bundled_code {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "jetbrains_mono".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "jetbrains_mono".to_owned());
            // Also add to Proportional as fallback so Nerd Font icons render in file tree
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push("jetbrains_mono".to_owned());
            break;
        }
    }

    // Fallback UI fonts (system)
    let ui_candidates: &[&str] = &[
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\segoeuisl.ttf",
        r"C:\Windows\Fonts\calibri.ttf",
    ];
    for path in ui_candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let name = format!("ui_{}", path.rsplit_once('\\').map(|(_, f)| f).unwrap_or(path));
            fonts.font_data.insert(
                name.clone(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push(name);
            break;
        }
    }

    // Fallback code fonts (system)
    let code_candidates: &[&str] = &[
        r"C:\Windows\Fonts\consola.ttf",
        r"C:\Windows\Fonts\cour.ttf",
        r"C:\Windows\Fonts\lucon.ttf",
    ];
    for path in code_candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let name = format!("code_{}", path.rsplit_once('\\').map(|(_, f)| f).unwrap_or(path));
            fonts.font_data.insert(
                name.clone(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push(name);
            break;
        }
    }

    // CJK fallback
    let cjk_candidates: &[&str] = &[
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    ];
    for path in cjk_candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let name = format!("cjk_{}", path.rsplit_once('/').or_else(|| path.rsplit_once('\\')).map(|(_, f)| f).unwrap_or(path));
            fonts.font_data.insert(
                name.clone(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push(name.clone());
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push(name);
            break;
        }
    }
}
