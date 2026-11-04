use eframe::egui::{
    self, Color32, CornerRadius, Frame, Margin, Stroke,
};

/// Colors used throughout the application.
#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub surface: Color32,
    pub field: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub accent_text: Color32,
    pub success: Color32,
    pub error: Color32,
}

impl Palette {
    /// Return the palette for the selected theme.
    pub fn new(dark: bool) -> Self {
        if dark {
            Self {
                background: Color32::from_rgb(14, 18, 29),
                surface: Color32::from_rgb(25, 31, 45),
                field: Color32::from_rgb(18, 23, 35),
                border: Color32::from_rgb(49, 60, 79),
                text: Color32::from_rgb(235, 240, 250),
                muted: Color32::from_rgb(151, 164, 184),
                accent: Color32::from_rgb(59, 130, 246),
                accent_text: Color32::WHITE,
                success: Color32::from_rgb(74, 222, 128),
                error: Color32::from_rgb(248, 113, 113),
            }
        } else {
            Self {
                background: Color32::from_rgb(245, 247, 252),
                surface: Color32::WHITE,
                field: Color32::from_rgb(237, 241, 247),
                border: Color32::from_rgb(218, 225, 235),
                text: Color32::from_rgb(29, 41, 57),
                muted: Color32::from_rgb(101, 115, 135),
                accent: Color32::from_rgb(37, 99, 235),
                accent_text: Color32::WHITE,
                success: Color32::from_rgb(22, 145, 88),
                error: Color32::from_rgb(210, 55, 70),
            }
        }
    }
}

/// Apply the selected theme to egui.
pub fn apply_theme(ctx: &egui::Context, dark: bool) {
    let palette = Palette::new(dark);

    ctx.set_theme(if dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });

    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    visuals.panel_fill = palette.background;
    visuals.window_fill = palette.surface;
    visuals.extreme_bg_color = palette.field;
    visuals.text_edit_bg_color = Some(palette.field);

    visuals.selection.bg_fill = palette.accent;
    visuals.selection.stroke =
        Stroke::new(1.0, palette.accent_text);

    visuals.widgets.inactive.corner_radius =
        CornerRadius::same(8);

    visuals.widgets.hovered.corner_radius =
        CornerRadius::same(8);

    visuals.widgets.active.corner_radius =
        CornerRadius::same(8);

    ctx.set_visuals(visuals);

    // Global spacing for a more comfortable layout.
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing =
            egui::vec2(10.0, 10.0);

        style.spacing.button_padding =
            egui::vec2(14.0, 9.0);

        style.spacing.interact_size.y = 34.0;
    });
}

/// Shared card style used by all main sections.
pub fn card(palette: &Palette) -> Frame {
    Frame::new()
        .fill(palette.surface)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(20))
}

/// Standard primary action button.
pub fn primary_button(
    label: &'static str,
    palette: &Palette,
) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(label)
            .strong()
            .color(palette.accent_text),
    )
        .fill(palette.accent)
        .corner_radius(8)
        .min_size(egui::vec2(165.0, 36.0))
}

/// Standard secondary action button.
pub fn secondary_button(
    label: &'static str,
) -> egui::Button<'static> {
    egui::Button::new(label)
        .corner_radius(8)
        .min_size(egui::vec2(95.0, 36.0))
}