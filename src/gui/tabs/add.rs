use std::path::PathBuf;
use std::fs;
use std::process::Command;
use eframe::egui;
use rfd::FileDialog;
use crate::gui::theme::*;
use crate::gui::tabs::{section_header, folder_button, primary_button, secondary_button};

/// Add tab - install mods, textures, shaders, and set username
pub struct AddTab {
    /// Username input
    username: String,
    /// Error message
    error_message: Option<String>,
    /// Success message
    success_message: Option<String>,
    /// Mods folder path
    mods_path: Option<PathBuf>,
    /// Resource packs folder path
    resource_packs_path: Option<PathBuf>,
    /// Shaders folder path
    shaders_path: Option<PathBuf>,
}

impl AddTab {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            error_message: None,
            success_message: None,
            mods_path: None,
            resource_packs_path: None,
            shaders_path: None,
        }
    }

    /// Initialize folder paths based on game directory
    fn init_paths(&mut self, game_dir: &Option<PathBuf>) {
        if let Some(game_dir) = game_dir {
            self.mods_path = Some(game_dir.join("mods"));
            self.resource_packs_path = Some(game_dir.join("resourcepacks"));
            self.shaders_path = Some(game_dir.join("shaderpacks"));
        }
    }

    /// Open folder in file explorer
    fn open_folder(&self, path: &Option<PathBuf>) {
        if let Some(path) = path {
            if path.exists() {
                let _ = Command::new("explorer").arg(path).spawn();
            }
        }
    }

    /// Create folder if it doesn't exist
    fn ensure_folder(&self, path: &Option<PathBuf>) -> Result<(), String> {
        if let Some(path) = path {
            fs::create_dir_all(path)
                .map_err(|e| format!("Erro ao criar pasta: {}", e))?;
        }
        Ok(())
    }

    /// Install mod (.jar or .hmod file)
    fn install_mod(&mut self, _game_dir: &Option<PathBuf>) {
        if let Some(path) = FileDialog::new()
            .add_filter("Mod files", &["jar", "hmod"])
            .set_title("Selecionar Mod para instalar")
            .pick_file()
        {
            if let Some(mods_dir) = &self.mods_path {
                let _ = self.ensure_folder(&self.mods_path);
                let dest = mods_dir.join(path.file_name().unwrap());
                match fs::copy(&path, &dest) {
                    Ok(_) => self.success_message = Some(format!("Mod instalado: {}", dest.display())),
                    Err(e) => self.error_message = Some(format!("Erro ao instalar mod: {}", e)),
                }
            }
        }
    }

    /// Install resource pack (.zip or folder)
    fn install_resource_pack(&mut self, _game_dir: &Option<PathBuf>) {
        if let Some(path) = FileDialog::new()
            .add_filter("Resource Pack", &["zip"])
            .set_title("Selecionar Resource Pack (.zip) ou pasta")
            .pick_file()
        {
            if let Some(rp_dir) = &self.resource_packs_path {
                let _ = self.ensure_folder(&self.resource_packs_path);
                let dest = rp_dir.join(path.file_name().unwrap());
                match fs::copy(&path, &dest) {
                    Ok(_) => self.success_message = Some(format!("Resource Pack instalado: {}", dest.display())),
                    Err(e) => self.error_message = Some(format!("Erro ao instalar resource pack: {}", e)),
                }
            }
        }
    }

    /// Install shader pack (.zip or folder)
    fn install_shader_pack(&mut self, _game_dir: &Option<PathBuf>) {
        if let Some(path) = FileDialog::new()
            .add_filter("Shader Pack", &["zip"])
            .set_title("Selecionar Shader Pack (.zip) ou pasta")
            .pick_file()
        {
            if let Some(shaders_dir) = &self.shaders_path {
                let _ = self.ensure_folder(&self.shaders_path);
                let dest = shaders_dir.join(path.file_name().unwrap());
                match fs::copy(&path, &dest) {
                    Ok(_) => self.success_message = Some(format!("Shader Pack instalado: {}", dest.display())),
                    Err(e) => self.error_message = Some(format!("Erro ao instalar shader pack: {}", e)),
                }
            }
        }
    }

    /// Draw a clickable link
    fn draw_link(&self, ui: &mut egui::Ui, text: &str, url: &str) {
        let response = ui.add(egui::Label::new(
            egui::RichText::new(text)
                .size(11.0)
                .color(BRAND_RED)
                .underline()
        ).sense(egui::Sense::click()));

        if response.hovered() {
            ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }

        if response.clicked() {
            let _ = open::that(url);
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
        // Initialize paths
        self.init_paths(resolved_game_dir);

        // Username section - pre-compute translated strings to avoid borrow issues
        let username_section_title = self.tr(config, "Username (Offline Mode)", "Nome de Usuário (Modo Offline)");
        let username_desc = self.tr(config, "Set the name that will be used in the game. In offline mode, any name can be used.", "Defina o nome que será usado no jogo. No modo offline, qualquer nome pode ser usado.");
        let name_label = self.tr(config, "Name:", "Nome:");
        let hint_text = self.tr(config, "Enter your name...", "Digite seu nome...");
        let save_name = self.tr(config, "Save Name", "Salvar Nome");
        let current_name_label = self.tr(config, "Current name", "Nome atual");
        let username_saved = self.tr(config, "Username saved!", "Nome de usuário salvo!");

        section_header(ui, username_section_title);

        ui.label(egui::RichText::new(username_desc).size(12.0).color(ICON_COLOR));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(name_label).size(13.0).color(ICON_COLOR));
            let response = ui.add_sized([250.0, 32.0], egui::TextEdit::singleline(&mut self.username)
                .hint_text(egui::RichText::new(hint_text).color(SEPARATOR_COLOR))
                .frame(true)
            );
            if response.changed() {
                config.player_name = self.username.clone();
            }
            ui.add_space(10.0);
            if primary_button(ui, save_name, !self.username.trim().is_empty()) {
                config.player_name = self.username.trim().to_string();
                self.success_message = Some(username_saved.to_string());
            }
        });

        if !config.player_name.is_empty() {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(format!("{}: {}", current_name_label, config.player_name)).size(11.0).color(egui::Color32::from_rgb(0x45, 0xC2, 0x7A)));
        }
        ui.add_space(20.0);

        // Mods section
        let mods_title = self.tr(config, "Mods", "Mods");
        let mods_desc = self.tr(config, "Install native Reset mods (.hmod) or Java mods (.jar compatible).", "Instale mods nativos do Reset (.hmod) ou mods Java (.jar compatíveis).");
        let install_mod = self.tr(config, "Install Mod (.jar/.hmod)", "Instalar Mod (.jar/.hmod)");
        let open_mods_folder = self.tr(config, "Open Mods Folder", "Abrir Pasta de Mods");
        let browse_mods = self.tr(config, "🔗 Browse Mods on playhorizon.dev/mods", "🔗 Ver Mods em playhorizon.dev/mods");
        let folder_label = self.tr(config, "Folder", "Pasta");
        let mods_installed = self.tr(config, "mod(s) installed", "mod(s) instalado(s)");
        let no_mods = self.tr(config, "No mods installed", "Nenhum mod instalado");

        section_header(ui, mods_title);

        ui.label(egui::RichText::new(mods_desc).size(12.0).color(ICON_COLOR));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if secondary_button(ui, install_mod, resolved_game_dir.is_some()) {
                self.install_mod(resolved_game_dir);
            }
            ui.add_space(10.0);
            if folder_button(ui, open_mods_folder, self.mods_path.as_ref().unwrap_or(&PathBuf::new()), self.mods_path.as_ref().map(|p| p.exists()).unwrap_or(false)) {
                self.open_folder(&self.mods_path);
            }
            ui.add_space(10.0);
            // Link to playhorizon.dev/mods
            self.draw_link(ui, browse_mods, "https://playhorizon.dev/mods");
        });

        if let Some(mods_path) = &self.mods_path {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(format!("{}: {}", folder_label, mods_path.display())).size(10.0).color(SEPARATOR_COLOR));
            if mods_path.exists() {
                let entries = fs::read_dir(mods_path).ok();
                if let Some(entries) = entries {
                    let count = entries.count();
                    if count > 0 {
                        ui.label(egui::RichText::new(format!("{} {}", count, mods_installed)).size(10.0).color(ICON_COLOR));
                    } else {
                        ui.label(egui::RichText::new(no_mods).size(10.0).color(SEPARATOR_COLOR));
                    }
                }
            }
        }
        ui.add_space(20.0);

        // Resource Packs section
        let rp_title = self.tr(config, "Resource Packs (Textures)", "Resource Packs (Texturas)");
        let rp_desc = self.tr(config, "Install texture packs (.zip) to change the appearance of blocks and items.", "Instale pacotes de texturas (.zip) para alterar a aparência dos blocos e itens.");
        let install_rp = self.tr(config, "Install Resource Pack (.zip)", "Instalar Resource Pack (.zip)");
        let open_rp_folder = self.tr(config, "Open Textures Folder", "Abrir Pasta de Texturas");
        let browse_rp = self.tr(config, "🔗 Browse Packs on playhorizon.dev/mods", "🔗 Ver Pacotes em playhorizon.dev/mods");
        let rp_installed = self.tr(config, "resource pack(s) installed", "resource pack(s) instalado(s)");
        let no_rp = self.tr(config, "No resource packs installed", "Nenhum resource pack instalado");

        section_header(ui, rp_title);

        ui.label(egui::RichText::new(rp_desc).size(12.0).color(ICON_COLOR));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if secondary_button(ui, install_rp, resolved_game_dir.is_some()) {
                self.install_resource_pack(resolved_game_dir);
            }
            ui.add_space(10.0);
            if folder_button(ui, open_rp_folder, self.resource_packs_path.as_ref().unwrap_or(&PathBuf::new()), self.resource_packs_path.as_ref().map(|p| p.exists()).unwrap_or(false)) {
                self.open_folder(&self.resource_packs_path);
            }
            ui.add_space(10.0);
            // Link to playhorizon.dev/mods
            self.draw_link(ui, browse_rp, "https://playhorizon.dev/mods");
        });

        if let Some(rp_path) = &self.resource_packs_path {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(format!("{}: {}", folder_label, rp_path.display())).size(10.0).color(SEPARATOR_COLOR));
            if rp_path.exists() {
                let entries = fs::read_dir(rp_path).ok();
                if let Some(entries) = entries {
                    let count = entries.count();
                    if count > 0 {
                        ui.label(egui::RichText::new(format!("{} {}", count, rp_installed)).size(10.0).color(ICON_COLOR));
                    } else {
                        ui.label(egui::RichText::new(no_rp).size(10.0).color(SEPARATOR_COLOR));
                    }
                }
            }
        }
        ui.add_space(20.0);

        // Shaders section
        let shaders_title = self.tr(config, "Shaders", "Shaders");
        let shaders_desc = self.tr(config, "Install shader packs (.zip) to improve lighting and visual effects.", "Instale pacotes de shaders (.zip) para melhorar a iluminação e efeitos visuais.");
        let install_shaders = self.tr(config, "Install Shader Pack (.zip)", "Instalar Shader Pack (.zip)");
        let open_shaders_folder = self.tr(config, "Open Shaders Folder", "Abrir Pasta de Shaders");
        let browse_shaders = self.tr(config, "🔗 Browse Shaders on playhorizon.dev/mods", "🔗 Ver Shaders em playhorizon.dev/mods");
        let shaders_installed = self.tr(config, "shader pack(s) installed", "shader pack(s) instalado(s)");
        let no_shaders = self.tr(config, "No shader packs installed", "Nenhum shader pack instalado");

        section_header(ui, shaders_title);

        ui.label(egui::RichText::new(shaders_desc).size(12.0).color(ICON_COLOR));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if secondary_button(ui, install_shaders, resolved_game_dir.is_some()) {
                self.install_shader_pack(resolved_game_dir);
            }
            ui.add_space(10.0);
            if folder_button(ui, open_shaders_folder, self.shaders_path.as_ref().unwrap_or(&PathBuf::new()), self.shaders_path.as_ref().map(|p| p.exists()).unwrap_or(false)) {
                self.open_folder(&self.shaders_path);
            }
            ui.add_space(10.0);
            // Link to playhorizon.dev/mods
            self.draw_link(ui, browse_shaders, "https://playhorizon.dev/mods");
        });

        if let Some(shaders_path) = &self.shaders_path {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(format!("{}: {}", folder_label, shaders_path.display())).size(10.0).color(SEPARATOR_COLOR));
            if shaders_path.exists() {
                let entries = fs::read_dir(shaders_path).ok();
                if let Some(entries) = entries {
                    let count = entries.count();
                    if count > 0 {
                        ui.label(egui::RichText::new(format!("{} {}", count, shaders_installed)).size(10.0).color(ICON_COLOR));
                    } else {
                        ui.label(egui::RichText::new(no_shaders).size(10.0).color(SEPARATOR_COLOR));
                    }
                }
            }
        }

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
        let info_title = self.tr(config, "Information", "Informações");
        let info_mods = self.tr(config, "• Mods: .hmod files (native Reset) or .jar (compatible)", "• Mods: arquivos .hmod (nativos Reset) ou .jar (compatíveis)");
        let info_rp = self.tr(config, "• Resource Packs: .zip files containing custom textures", "• Resource Packs: arquivos .zip contendo texturas personalizadas");
        let info_shaders = self.tr(config, "• Shaders: .zip files with GLSL shaders for visual effects", "• Shaders: arquivos .zip com shaders GLSL para efeitos visuais");
        let info_username = self.tr(config, "• Username: any name works in offline mode", "• Nome de usuário: qualquer nome funciona no modo offline");

        section_header(ui, info_title);

        ui.label(egui::RichText::new(info_mods).size(11.0).color(ICON_COLOR));
        ui.add_space(4.0);
        ui.label(egui::RichText::new(info_rp).size(11.0).color(ICON_COLOR));
        ui.add_space(4.0);
        ui.label(egui::RichText::new(info_shaders).size(11.0).color(ICON_COLOR));
        ui.add_space(4.0);
        ui.label(egui::RichText::new(info_username).size(11.0).color(ICON_COLOR));
    }
}

impl Default for AddTab {
    fn default() -> Self {
        Self::new()
    }
}