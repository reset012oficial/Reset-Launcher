use eframe::egui;
use crate::gui::theme::*;
use crate::gui::tabs::{section_header, labeled_input, labeled_number_input, labeled_checkbox};

/// Settings tab - instance settings (memory, window size, fullscreen), language, and credits
pub struct SettingsTab {
    /// Error message
    error_message: Option<String>,
    /// Success message
    success_message: Option<String>,
}

impl SettingsTab {
    pub fn new() -> Self {
        Self {
            error_message: None,
            success_message: None,
        }
    }

    /// Get translated text based on current language
    fn tr<'a>(&self, config: &crate::gui::app::LauncherConfig, en: &'a str, pt: &'a str) -> &'a str {
        if config.language == "en" { en } else { pt }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut crate::gui::app::LauncherConfig,
    ) {
        // Pre-compute all translated strings to avoid borrow issues
        let instance_settings = self.tr(config, "Instance Settings", "Configurações da Instância");
        let language_label = self.tr(config, "Language", "Idioma");
        let ram_label = self.tr(config, "RAM Memory (MB)", "Memória RAM (MB)");
        let ram_desc = self.tr(config, "Memory allocated for Reset. Recommended: 4096 MB or more.", "Memória alocada para o Reset. Recomendado: 4096 MB ou mais.");
        let window_size = self.tr(config, "Window Size", "Tamanho da Janela");
        let width_label = self.tr(config, "Width", "Largura");
        let height_label = self.tr(config, "Height", "Altura");
        let fullscreen = self.tr(config, "Fullscreen", "Tela Cheia");
        let fullscreen_desc = self.tr(config, "Start the game in fullscreen mode.", "Inicia o jogo em modo de tela cheia.");
        let render_distance = self.tr(config, "Render Distance (chunks)", "Distância de Renderização (chunks)");
        let render_desc = self.tr(config, "Number of chunks rendered around the player. Higher values require more performance.", "Número de chunks renderizados ao redor do jogador. Valores maiores exigem mais desempenho.");
        let world_settings = self.tr(config, "World Settings", "Configurações do Mundo");
        let world_name = self.tr(config, "World Name", "Nome do Mundo");
        let world_name_hint = self.tr(config, "Leave empty for last world", "Deixe vazio para último mundo");
        let world_seed = self.tr(config, "World Seed", "Seed do Mundo");
        let world_seed_hint = self.tr(config, "Optional", "Opcional");
        let creative_mode = self.tr(config, "Creative Mode", "Modo Criativo");
        let server_connection = self.tr(config, "Server Connection", "Conexão com Servidor");
        let server_address = self.tr(config, "Server Address", "Endereço do Servidor");
        let server_hint = self.tr(config, "host:port (e.g. localhost:25565)", "host:porta (ex: localhost:25565)");
        let server_desc = self.tr(config, "Leave empty for singleplayer. Works only with offline-mode servers.", "Deixe vazio para jogar singleplayer. Funciona apenas com servidores em modo offline.");
        let headless = self.tr(config, "Headless Mode (no window)", "Modo Headless (sem janela)");
        let headless_desc = self.tr(config, "Run the game without graphical interface (useful for servers).", "Executa o jogo sem interface gráfica (útil para servidores).");
        let credits_title = self.tr(config, "Credits", "Créditos");
        let created_by = self.tr(config, "Created by reset012", "Criado por reset012");
        let horizon_desc1 = self.tr(config, "A clean-room rewrite of Minecraft Java Edition in Rust", "Uma reescritura limpa (clean-room) do Minecraft Java Edition em Rust");
        let horizon_desc2 = self.tr(config, "Developed by the Horizon team", "Desenvolvido pela equipe do Horizon");
        let minecraft_by = self.tr(config, "Created by Mojang Studios", "Criado por Mojang Studios");
        let minecraft_rights = self.tr(config, "All Minecraft rights belong to Mojang/Microsoft", "Todos os direitos do Minecraft pertencem à Mojang/Microsoft");
        let claude_desc = self.tr(config, "AI assistant from Anthropic that helped develop this launcher", "Assistente de IA da Anthropic que auxiliou no desenvolvimento deste launcher");
        let libs_title = self.tr(config, "Libraries and Tools Used", "Bibliotecas e Ferramentas Utilizadas");

        // Instance Settings section
        section_header(ui, instance_settings);

        // Language selector
        ui.horizontal(|ui| {
            ui.add_sized([140.0, 0.0], egui::Label::new(
                egui::RichText::new(language_label).size(13.0).color(ICON_COLOR)
            ));
            egui::ComboBox::from_id_salt("language_selector")
                .selected_text(match config.language.as_str() {
                    "en" => "English",
                    "pt" => "Português",
                    _ => "Português",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut config.language, "en".to_string(), "English");
                    ui.selectable_value(&mut config.language, "pt".to_string(), "Português");
                });
        });
        ui.add_space(12.0);

        // Memory allocation
        labeled_number_input(ui, ram_label, &mut config.memory_mb, 1024, 16384);
        ui.label(egui::RichText::new(ram_desc).size(11.0).color(ICON_COLOR));
        ui.add_space(12.0);

        // Window size
        ui.label(egui::RichText::new(window_size).size(13.0).strong().color(WHITE));
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            labeled_number_input(ui, width_label, &mut config.window_width, 640, 3840);
            labeled_number_input(ui, height_label, &mut config.window_height, 480, 2160);
        });

        ui.add_space(6.0);

        // Fullscreen checkbox
        labeled_checkbox(ui, fullscreen, &mut config.fullscreen);
        ui.label(egui::RichText::new(fullscreen_desc).size(11.0).color(ICON_COLOR));
        ui.add_space(12.0);

        // Render distance
        labeled_number_input(ui, render_distance, &mut config.render_distance, 2, 128);
        ui.label(egui::RichText::new(render_desc).size(11.0).color(ICON_COLOR));
        ui.add_space(12.0);

        // World settings
        ui.label(egui::RichText::new(world_settings).size(13.0).strong().color(WHITE));
        ui.add_space(6.0);

        labeled_input(ui, world_name, &mut config.world_name, world_name_hint);
        labeled_input(ui, world_seed, &mut config.seed_string, world_seed_hint);
        labeled_checkbox(ui, creative_mode, &mut config.creative);

        // Server connection
        ui.add_space(12.0);
        ui.label(egui::RichText::new(server_connection).size(13.0).strong().color(WHITE));
        ui.add_space(6.0);
        labeled_input(ui, server_address, &mut config.server_address, server_hint);
        ui.label(egui::RichText::new(server_desc).size(11.0).color(ICON_COLOR));
        ui.add_space(12.0);

        // Headless mode
        labeled_checkbox(ui, headless, &mut config.headless);
        ui.label(egui::RichText::new(headless_desc).size(11.0).color(ICON_COLOR));
        ui.add_space(20.0);

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

        // Credits section
        section_header(ui, credits_title);

        // Credits content in a scrollable area
        ui.vertical(|ui| {
            ui.set_max_width(700.0);

            // Reset Launcher
            ui.label(egui::RichText::new("Reset Launcher").size(16.0).strong().color(BRAND_RED));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(created_by).size(12.0).color(WHITE));
            ui.add_space(12.0);

            // Horizon
            ui.label(egui::RichText::new("Horizon (Minecraft Rust Edition)").size(16.0).strong().color(WHITE));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(horizon_desc1).size(12.0).color(ICON_COLOR));
            ui.label(egui::RichText::new(horizon_desc2).size(12.0).color(ICON_COLOR));
            ui.add_space(12.0);

            // Minecraft
            ui.label(egui::RichText::new("Minecraft").size(16.0).strong().color(WHITE));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(minecraft_by).size(12.0).color(ICON_COLOR));
            ui.label(egui::RichText::new(minecraft_rights).size(12.0).color(ICON_COLOR));
            ui.add_space(12.0);

            // Claude Code
            ui.label(egui::RichText::new("Claude Code").size(16.0).strong().color(WHITE));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(claude_desc).size(12.0).color(ICON_COLOR));
            ui.add_space(12.0);

            // Libraries/Tools
            ui.label(egui::RichText::new(libs_title).size(14.0).strong().color(WHITE));
            ui.add_space(8.0);

            let libraries = vec![
                ("Rust", self.tr(config, "Programming language", "Linguagem de programação")),
                ("eframe / egui", self.tr(config, "Immediate mode GUI framework", "Framework de interface gráfica imediata")),
                ("tokio", self.tr(config, "Async runtime for Rust", "Runtime assíncrono para Rust")),
                ("serde / serde_json", self.tr(config, "Serialization and deserialization", "Serialização e desserialização")),
                ("clap", self.tr(config, "Command line parser", "Parser de linha de comando")),
                ("rfd", self.tr(config, "Native file dialogs", "Diálogos de arquivo nativos")),
                ("directories-next", self.tr(config, "System standard directories", "Diretórios padrão do sistema")),
                ("image", self.tr(config, "Image loading and processing", "Carregamento e processamento de imagens")),
            ];

            for (name, desc) in libraries {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("•").size(12.0).color(ICON_COLOR));
                    ui.label(egui::RichText::new(name).size(12.0).strong().color(WHITE));
                    ui.label(egui::RichText::new("-").size(12.0).color(SEPARATOR_COLOR));
                    ui.label(egui::RichText::new(desc).size(12.0).color(ICON_COLOR));
                });
                ui.add_space(4.0);
            }
        });
    }
}

impl Default for SettingsTab {
    fn default() -> Self {
        Self::new()
    }
}