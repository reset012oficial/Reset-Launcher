use std::path::{Path, PathBuf};
use std::fs;
use eframe::egui;
use rfd::FileDialog;
use image::GenericImageView;
use crate::gui::theme::*;
use crate::gui::tabs::{section_header, primary_button, secondary_button};

// Calculate offline UUID from username (Minecraft's algorithm: UUID.nameUUIDFromBytes("OfflinePlayer:" + username))
fn calculate_offline_uuid(username: &str) -> String {
    use md5::compute;
    let input = format!("OfflinePlayer:{}", username);
    let hash = compute(input.as_bytes());
    let bytes = hash.0; // md5::Digest is a [u8; 16]

    // Format as UUID version 3 (name-based, MD5)
    // Set version (4 bits) to 3
    let mut b = bytes;
    b[6] = (b[6] & 0x0F) | 0x30;
    // Set variant (2 bits) to 10 (RFC 4122)
    b[8] = (b[8] & 0x3F) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3],
        b[4], b[5],
        b[6], b[7],
        b[8], b[9],
        b[10], b[11], b[12], b[13], b[14], b[15]
    )
}

/// Skin tab - allows changing the player skin via a .png file
pub struct SkinTab {
    /// Selected skin path
    skin_path: Option<PathBuf>,
    /// Skin preview texture
    skin_texture: Option<egui::TextureHandle>,
    /// Error message
    error_message: Option<String>,
    /// Success message
    success_message: Option<String>,
}

impl SkinTab {
    pub fn new() -> Self {
        Self {
            skin_path: None,
            skin_texture: None,
            error_message: None,
            success_message: None,
        }
    }

    /// Load and validate skin image
    fn load_skin(&mut self, ctx: &egui::Context, path: &Path) -> Result<(), String> {
        // Check file extension
        if path.extension().map_or(true, |ext| ext.to_ascii_lowercase() != "png") {
            return Err("File must be a .png".to_string());
        }

        // Read file
        let image_bytes = fs::read(path)
            .map_err(|e| format!("Error reading file: {}", e))?;

        // Load image to verify it's valid and get dimensions
        let image = image::load_from_memory(&image_bytes)
            .map_err(|e| format!("Error loading image: {}", e))?;

        // Check dimensions (Minecraft skins are typically 64x32 or 64x64)
        let (width, height) = image.dimensions();
        if width != 64 || (height != 32 && height != 64) {
            return Err(format!("Invalid dimensions: {}x{} (expected 64x32 or 64x64)", width, height));
        }

        // Create egui texture
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &image.to_rgba8(),
        );

        let texture = ctx.load_texture(
            "skin_preview",
            color_image,
            egui::TextureOptions::LINEAR,
        );

        self.skin_texture = Some(texture);
        self.skin_path = Some(path.to_path_buf());
        self.error_message = None;
        self.success_message = Some("Skin loaded successfully!".to_string());

        Ok(())
    }

    /// Apply skin to game directory
    fn apply_skin(&mut self, game_dir: &Option<PathBuf>, username: &str) -> Result<(), String> {
        let skin_path = self.skin_path.as_ref().ok_or("No skin selected")?;
        let game_dir = game_dir.as_ref().ok_or("Game directory not set")?;

        // Create skins directory if it doesn't exist
        let skins_dir = game_dir.join("skins");
        fs::create_dir_all(&skins_dir)
            .map_err(|e| format!("Error creating skins folder: {}", e))?;

        // Read skin bytes once
        let skin_bytes = fs::read(skin_path)
            .map_err(|e| format!("Error reading skin file: {}", e))?;

        // Save as steve.png (fallback/default)
        let steve_path = skins_dir.join("steve.png");
        fs::write(&steve_path, &skin_bytes)
            .map_err(|e| format!("Error copying skin as steve.png: {}", e))?;

        // Also save as <uuid>.png using the offline UUID from username
        if !username.trim().is_empty() {
            let uuid = calculate_offline_uuid(username.trim());
            let uuid_path = skins_dir.join(format!("{}.png", uuid));
            fs::write(&uuid_path, &skin_bytes)
                .map_err(|e| format!("Error copying skin as {}.png: {}", uuid, e))?;

            self.success_message = Some(format!("Skin applied as steve.png and {}.png", uuid));
        } else {
            self.success_message = Some(format!("Skin applied as steve.png (set username in Add tab to use UUID)"));
        }

        Ok(())
    }

    /// Select skin file
    fn select_skin(&mut self, ctx: &egui::Context) {
        if let Some(path) = FileDialog::new()
            .add_filter("PNG images", &["png"])
            .set_title("Select Player Skin (.png)")
            .pick_file()
        {
            self.error_message = None;
            self.success_message = None;
            if let Err(e) = self.load_skin(ctx, &path) {
                self.error_message = Some(e);
            }
        }
    }

    /// Get translated text based on config language
    fn tr<'a>(&self, config: &crate::gui::app::LauncherConfig, en: &'a str, pt: &'a str) -> &'a str {
        if config.language == "en" { en } else { pt }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut crate::gui::app::LauncherConfig,
        resolved_game_dir: &Option<PathBuf>,
    ) {
        // Skin selection section
        section_header(ui, self.tr(config, "Player Skin", "Skin do Jogador"));

        ui.label(egui::RichText::new(self.tr(config, "Select a .png file to use as player skin. Minecraft skins must be 64x32 (classic format) or 64x64 pixels (new format with layers).", "Selecione um arquivo .png para usar como skin do jogador. As skins do Minecraft devem ter 64x32 (formato clássico) ou 64x64 pixels (formato novo com camadas).")).size(12.0).color(ICON_COLOR));
        ui.add_space(12.0);

        // Skin preview area
        ui.horizontal(|ui| {
            // Preview on the left
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(self.tr(config, "Preview", "Pré-visualização")).size(13.0).strong().color(WHITE));
                ui.add_space(6.0);

                let preview_size = 128.0;
                let (rect, _response) = ui.allocate_exact_size(
                    egui::vec2(preview_size, preview_size),
                    egui::Sense::hover(),
                );

                // Draw background for preview
                ui.painter().rect_filled(rect, 8.0, egui::Color32::from_rgb(0x1A, 0x1A, 0x1A));
                ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(1.0_f32, SEPARATOR_COLOR));

                // Draw skin texture if available
                if let Some(texture) = &self.skin_texture {
                    let texture_id = texture.id();
                    ui.painter().image(
                        texture_id,
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        WHITE,
                    );
                } else {
                    // Placeholder
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        self.tr(config, "No skin\nselected", "Nenhuma skin\nselecionada"),
                        egui::FontId::new(12.0, egui::FontFamily::Proportional),
                        ICON_COLOR,
                    );
                }
            });

            ui.add_space(20.0);

            // Controls on the right
            ui.vertical(|ui| {
                ui.add_space(20.0);

                // Select skin button
                if secondary_button(ui, self.tr(config, "Select Skin (.png)", "Selecionar Skin (.png)"), true) {
                    self.select_skin(ui.ctx());
                }
                ui.add_space(10.0);

                // Apply skin button
                if primary_button(ui, self.tr(config, "Apply Skin", "Aplicar Skin"), self.skin_path.is_some() && resolved_game_dir.is_some()) {
                    if let Err(e) = self.apply_skin(resolved_game_dir, &config.player_name) {
                        self.error_message = Some(e);
                        self.success_message = None;
                    }
                }

                // Show current skin path
                if let Some(path) = &self.skin_path {
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new(self.tr(config, "Selected file:", "Arquivo selecionado:")).size(11.0).color(ICON_COLOR));
                    ui.label(egui::RichText::new(path.display().to_string()).size(10.0).color(WHITE));
                }

                ui.add_space(10.0);

                // Reset button
                if self.skin_path.is_some() {
                    if secondary_button(ui, self.tr(config, "Remove Skin", "Remover Skin"), true) {
                        self.skin_path = None;
                        self.skin_texture = None;
                        self.error_message = None;
                        self.success_message = None;
                    }
                }
            });
        });

        // Error/Success messages
        if let Some(error) = &self.error_message {
            ui.add_space(16.0);
            ui.label(egui::RichText::new(error).size(12.0).color(BRAND_RED));
        }

        if let Some(success) = &self.success_message {
            ui.add_space(16.0);
            ui.label(egui::RichText::new(success).size(12.0).color(egui::Color32::from_rgb(0x45, 0xC2, 0x7A)));
        }

        ui.add_space(20.0);

        // Info section
        section_header(ui, self.tr(config, "Information", "Informações"));

        ui.label(egui::RichText::new(self.tr(config, "• Skins must be valid .png files", "• Skins devem ser arquivos .png válidos")).size(11.0).color(ICON_COLOR));
        ui.add_space(4.0);
        ui.label(egui::RichText::new(self.tr(config, "• Accepted dimensions: 64x32 (classic) or 64x64 (with layers)", "• Dimensões aceitas: 64x32 (clássico) ou 64x64 (com camadas)")).size(11.0).color(ICON_COLOR));
        ui.add_space(4.0);
        ui.label(egui::RichText::new(self.tr(config, "• Skin will be saved in the game directory's 'skins' folder as 'steve.png' AND '<uuid>.png'", "• A skin será salva na pasta 'skins' do diretório do jogo como 'steve.png' E '<uuid>.png'")).size(11.0).color(ICON_COLOR));
        ui.add_space(4.0);
        ui.label(egui::RichText::new(self.tr(config, "• UUID is calculated from username (offline mode: OfflinePlayer:<name>)", "• O UUID é calculado do nome de usuário (modo offline: OfflinePlayer:<nome>)")).size(11.0).color(ICON_COLOR));
        ui.add_space(4.0);
        ui.label(egui::RichText::new(self.tr(config, "• Set username in Add tab before applying skin", "• Defina o nome de usuário na aba Add antes de aplicar a skin")).size(11.0).color(ICON_COLOR));
        ui.add_space(4.0);
        ui.label(egui::RichText::new(self.tr(config, "• Works only in offline mode (no Mojang authentication)", "• Funciona apenas no modo offline (sem autenticação Mojang)")).size(11.0).color(ICON_COLOR));
    }
}

impl Default for SkinTab {
    fn default() -> Self {
        Self::new()
    }
}