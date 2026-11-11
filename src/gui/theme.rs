use eframe::egui;

/// Reset Launcher color theme - Exact specification colors
///
/// Palette:
/// - Titlebar/Sidebar background: #484848 (RGB: 72, 72, 72)
/// - Main content background: #252525 (RGB: 37, 37, 37)
/// - Title text: #FFFFFF
/// - Brand star: #E00018
/// - Icons: #858585
/// - Separators: #777777

pub fn theme() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    // Exact specification colors
    let _titlebar_bg = egui::Color32::from_rgb(0x48, 0x48, 0x48);  // #484848
    let _sidebar_bg = egui::Color32::from_rgb(0x48, 0x48, 0x48);   // #484848
    let main_bg = egui::Color32::from_rgb(0x25, 0x25, 0x25);       // #252525
    let icon_color = egui::Color32::from_rgb(0x85, 0x85, 0x85);    // #858585
    let _separator_color = egui::Color32::from_rgb(0x77, 0x77, 0x77); // #777777
    let white = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);          // #FFFFFF
    let brand_red = egui::Color32::from_rgb(0xE0, 0x00, 0x18);      // #E00018

    // Window fill - main content area color
    visuals.window_fill = main_bg;
    visuals.panel_fill = main_bg;
    visuals.window_shadow = egui::epaint::Shadow::NONE;

    // Widgets - use main_bg as base
    visuals.widgets.noninteractive.bg_fill = main_bg;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, white);
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(0.0);
    visuals.widgets.noninteractive.expansion = 0.0;

    visuals.widgets.inactive.bg_fill = main_bg;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, icon_color);
    visuals.widgets.inactive.rounding = egui::Rounding::same(0.0);
    visuals.widgets.inactive.expansion = 0.0;

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x3A, 0x3A, 0x3A); // Slightly lighter for hover
    visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, white);
    visuals.widgets.hovered.rounding = egui::Rounding::same(0.0);
    visuals.widgets.hovered.expansion = 0.0;

    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0x50, 0x50, 0x50);
    visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, white);
    visuals.widgets.active.rounding = egui::Rounding::same(0.0);
    visuals.widgets.active.expansion = 0.0;

    visuals.widgets.open.bg_fill = main_bg;
    visuals.widgets.open.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0_f32, white);
    visuals.widgets.open.rounding = egui::Rounding::same(0.0);
    visuals.widgets.open.expansion = 0.0;

    // Selection
    visuals.selection.bg_fill = brand_red;
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, brand_red);

    // Hyperlink
    visuals.hyperlink_color = brand_red;

    // Text colors
    visuals.override_text_color = Some(white);

    // Other
    visuals.window_rounding = egui::Rounding::same(0.0);
    visuals.menu_rounding = egui::Rounding::same(0.0);
    visuals.code_bg_color = main_bg;
    visuals.warn_fg_color = egui::Color32::from_rgb(0xF0, 0xB8, 0x4F);
    visuals.error_fg_color = brand_red;

    // Extensions
    visuals.extreme_bg_color = main_bg;
    visuals.faint_bg_color = main_bg;

    visuals
}

/// Color constants for easy access
pub const TITLEBAR_BG: egui::Color32 = egui::Color32::from_rgb(0x48, 0x48, 0x48); // #484848
pub const SIDEBAR_BG: egui::Color32 = egui::Color32::from_rgb(0x48, 0x48, 0x48);  // #484848
pub const MAIN_BG: egui::Color32 = egui::Color32::from_rgb(0x25, 0x25, 0x25);     // #252525
pub const ICON_COLOR: egui::Color32 = egui::Color32::from_rgb(0x85, 0x85, 0x85);   // #858585
pub const SEPARATOR_COLOR: egui::Color32 = egui::Color32::from_rgb(0x77, 0x77, 0x77); // #777777
pub const WHITE: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);        // #FFFFFF
pub const BRAND_RED: egui::Color32 = egui::Color32::from_rgb(0xE0, 0x00, 0x18);    // #E00018

/// Titlebar height in pixels
pub const TITLEBAR_HEIGHT: f32 = 44.0;

/// Sidebar width in pixels
pub const SIDEBAR_WIDTH: f32 = 45.0;

/// Main content top corner radius
pub const MAIN_CORNER_RADIUS: f32 = 22.0;