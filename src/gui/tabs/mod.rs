pub mod home;
pub mod skin;
pub mod add;
pub mod settings;

pub use home::HomeTab;
pub use skin::SkinTab;
pub use add::AddTab;
pub use settings::SettingsTab;

use eframe::egui;
use crate::gui::theme::*;

/// Helper function to draw a section header
pub fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(title).size(16.0).strong().color(WHITE));
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(8.0);
}

/// Helper function to draw a labeled input field
pub fn labeled_input(ui: &mut egui::Ui, label: &str, value: &mut String, hint: &str) {
    ui.horizontal(|ui| {
        ui.add_sized([140.0, 0.0], egui::Label::new(
            egui::RichText::new(label).size(13.0).color(ICON_COLOR)
        ));
        let response = ui.add_sized([300.0, 32.0], egui::TextEdit::singleline(value)
            .hint_text(egui::RichText::new(hint).color(SEPARATOR_COLOR))
            .frame(true)
        );
        if response.changed() {
            // Value changed
        }
    });
    ui.add_space(8.0);
}

/// Helper function to draw a labeled number input
pub fn labeled_number_input(ui: &mut egui::Ui, label: &str, value: &mut u32, min: u32, max: u32) {
    ui.horizontal(|ui| {
        ui.add_sized([140.0, 0.0], egui::Label::new(
            egui::RichText::new(label).size(13.0).color(ICON_COLOR)
        ));
        let mut val = *value as i64;
        let _response = ui.add(egui::DragValue::new(&mut val).speed(1.0).range(min as i64..=max as i64).suffix(""));
        *value = val as u32;
    });
    ui.add_space(8.0);
}

/// Helper function to draw a labeled checkbox
pub fn labeled_checkbox(ui: &mut egui::Ui, label: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.add_sized([140.0, 0.0], egui::Label::new(
            egui::RichText::new(label).size(13.0).color(ICON_COLOR)
        ));
        ui.checkbox(value, "");
    });
    ui.add_space(8.0);
}

/// Helper function to draw a folder button
pub fn folder_button(ui: &mut egui::Ui, label: &str, _path: &std::path::Path, enabled: bool) -> bool {
    let response = ui.add_enabled(enabled, egui::Button::new(
        egui::RichText::new(label).size(13.0).color(WHITE)
    ).fill(egui::Color32::from_rgb(0x3A, 0x3A, 0x3A))
    .stroke(egui::Stroke::NONE)
    .rounding(egui::Rounding::same(6.0))
    .min_size(egui::Vec2::new(180.0, 40.0)));

    if response.hovered() {
        ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    response.clicked()
}

/// Helper function to draw a primary action button (red accent)
pub fn primary_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> bool {
    let response = ui.add_enabled(enabled, egui::Button::new(
        egui::RichText::new(label).size(14.0).strong().color(WHITE)
    ).fill(BRAND_RED)
    .stroke(egui::Stroke::NONE)
    .rounding(egui::Rounding::same(6.0))
    .min_size(egui::Vec2::new(200.0, 44.0)));

    if response.hovered() && enabled {
        ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    response.clicked()
}

/// Helper function to draw a secondary button
pub fn secondary_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> bool {
    let response = ui.add_enabled(enabled, egui::Button::new(
        egui::RichText::new(label).size(13.0).color(WHITE)
    ).fill(egui::Color32::from_rgb(0x3A, 0x3A, 0x3A))
    .stroke(egui::Stroke::new(1.0_f32, SEPARATOR_COLOR))
    .rounding(egui::Rounding::same(6.0))
    .min_size(egui::Vec2::new(160.0, 36.0)));

    if response.hovered() && enabled {
        ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    response.clicked()
}

/// Helper function to draw a status indicator
pub fn status_indicator(ui: &mut egui::Ui, label: &str, status: bool) {
    ui.horizontal(|ui| {
        let color = if status { egui::Color32::from_rgb(0x45, 0xC2, 0x7A) } else { BRAND_RED };
        ui.label(egui::RichText::new("●").size(12.0).color(color));
        ui.label(egui::RichText::new(label).size(12.0).color(ICON_COLOR));
    });
    ui.add_space(4.0);
}