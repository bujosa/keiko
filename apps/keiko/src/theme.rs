//! Sumi ink, washi paper and a vermilion seal. Every colour the UI paints comes from [`Tokens`].

use std::sync::Arc;

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals};

#[derive(Debug, Clone, Copy)]
pub struct Tokens {
    pub bg: Color32,
    pub panel: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub faint: Color32,
    /// Vermilion: the seal, the brand.
    pub seal: Color32,
    pub amber: Color32,
    pub good: Color32,
    pub bad: Color32,
    /// Primary buttons: paper on ink (dark) or ink on paper (light).
    pub primary_bg: Color32,
    pub primary_text: Color32,
    pub dark: bool,
}

impl Tokens {
    pub const DARK: Tokens = Tokens {
        bg: Color32::from_rgb(14, 15, 18),
        panel: Color32::from_rgb(17, 18, 22),
        card: Color32::from_rgb(17, 18, 22),
        card_hover: Color32::from_rgb(24, 26, 32),
        border: Color32::from_rgb(38, 41, 49),
        border_strong: Color32::from_rgb(64, 68, 79),
        text: Color32::from_rgb(236, 232, 223),
        muted: Color32::from_rgb(150, 150, 154),
        faint: Color32::from_rgb(98, 99, 106),
        seal: Color32::from_rgb(235, 82, 66),
        amber: Color32::from_rgb(245, 184, 61),
        good: Color32::from_rgb(110, 207, 151),
        bad: Color32::from_rgb(248, 113, 113),
        primary_bg: Color32::from_rgb(244, 240, 231),
        primary_text: Color32::from_rgb(14, 15, 18),
        dark: true,
    };

    pub const LIGHT: Tokens = Tokens {
        bg: Color32::from_rgb(243, 239, 230),
        panel: Color32::from_rgb(237, 232, 221),
        card: Color32::from_rgb(250, 248, 242),
        card_hover: Color32::WHITE,
        border: Color32::from_rgb(218, 211, 196),
        border_strong: Color32::from_rgb(180, 172, 155),
        text: Color32::from_rgb(22, 23, 27),
        muted: Color32::from_rgb(98, 96, 92),
        faint: Color32::from_rgb(150, 146, 138),
        seal: Color32::from_rgb(196, 50, 38),
        amber: Color32::from_rgb(170, 105, 0),
        good: Color32::from_rgb(22, 128, 74),
        bad: Color32::from_rgb(200, 40, 40),
        primary_bg: Color32::from_rgb(22, 23, 27),
        primary_text: Color32::from_rgb(250, 248, 242),
        dark: false,
    };
}

pub fn display() -> FontFamily {
    FontFamily::Name("display".into())
}

pub fn mono() -> FontFamily {
    FontFamily::Name("mono".into())
}

pub fn mono_bold() -> FontFamily {
    FontFamily::Name("mono-bold".into())
}

/// Space Grotesk for names, JetBrains Mono for labels, Inter for reading. egui's own fonts stay
/// behind them for symbols the subset fonts lack (arrows).
pub fn fonts(ctx: &egui::Context) {
    let mut f = FontDefinitions::default();
    let fallback: Vec<String> = f.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for (name, bytes) in [
        ("inter", &include_bytes!("../../../assets/fonts/Inter-Regular.ttf")[..]),
        ("space-bold", &include_bytes!("../../../assets/fonts/SpaceGrotesk-Bold.ttf")[..]),
        ("space-medium", &include_bytes!("../../../assets/fonts/SpaceGrotesk-Medium.ttf")[..]),
        ("jbm", &include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf")[..]),
        ("jbm-bold", &include_bytes!("../../../assets/fonts/JetBrainsMono-Bold.ttf")[..]),
    ] {
        f.font_data.insert(name.into(), Arc::new(FontData::from_static(bytes)));
    }
    let with = |first: &str| {
        let mut v = vec![first.to_string()];
        v.extend(fallback.iter().cloned());
        v
    };
    f.families.insert(FontFamily::Proportional, with("inter"));
    f.families.insert(display(), with("space-bold"));
    f.families.insert(FontFamily::Name("display-medium".into()), with("space-medium"));
    f.families.insert(mono(), with("jbm"));
    f.families.insert(mono_bold(), with("jbm-bold"));
    let mut m = vec!["jbm".to_string()];
    m.extend(f.families.get(&FontFamily::Monospace).cloned().unwrap_or_default());
    f.families.insert(FontFamily::Monospace, m);
    ctx.set_fonts(f);
}

pub fn apply(ctx: &egui::Context, t: &Tokens) {
    let mut v = if t.dark { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = t.bg;
    v.window_fill = t.card;
    v.extreme_bg_color = if t.dark { Color32::from_rgb(10, 11, 13) } else { Color32::WHITE };
    v.faint_bg_color = t.card_hover;
    v.override_text_color = Some(t.text);
    v.hyperlink_color = t.seal;
    v.selection.bg_fill = t.seal.gamma_multiply(0.35);
    v.selection.stroke = Stroke::new(1.0, t.seal);
    v.window_corner_radius = egui::CornerRadius::same(2);
    v.window_stroke = Stroke::new(1.0, t.border_strong);
    v.menu_corner_radius = egui::CornerRadius::same(2);
    for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = egui::CornerRadius::same(2);
    }
    v.widgets.inactive.weak_bg_fill = v.extreme_bg_color;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.border);
    v.widgets.hovered.weak_bg_fill = t.card_hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.border_strong);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.border);
    let theme = if t.dark { egui::Theme::Dark } else { egui::Theme::Light };
    ctx.set_theme(theme);
    ctx.set_visuals_of(theme, v);
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(12.0, 7.0);
        s.spacing.interact_size.y = 34.0;
        s.text_styles.insert(TextStyle::Heading, FontId::new(24.0, display()));
        s.text_styles.insert(TextStyle::Body, FontId::new(15.0, FontFamily::Proportional));
        s.text_styles.insert(TextStyle::Button, FontId::new(13.0, mono_bold()));
        s.text_styles.insert(TextStyle::Monospace, FontId::new(13.0, mono()));
        s.text_styles.insert(TextStyle::Small, FontId::new(12.0, FontFamily::Proportional));
    });
}
