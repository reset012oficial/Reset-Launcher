use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use eframe::egui;
use rfd::FileDialog;
use crate::gui::theme::*;
use crate::gui::tabs::{section_header, primary_button, secondary_button};
use crate::gui::app::LaunchStatus;

/// Home tab - shows requirements check, server.jar download, and play button
pub struct HomeTab {
    /// Whether we're checking requirements
    checking: bool,
    /// Java found status
    java_found: bool,
    /// Java version string
    java_version: String,
    /// Reset client built status
    client_built: bool,
    /// Path to the built client binary (if found)
    client_binary_path: Option<PathBuf>,
    /// Resources prepared status
    resources_ready: bool,
    /// Server.jar selected path (downloaded)
    server_jar_path: Option<PathBuf>,
    /// Error message for server.jar
    server_jar_error: Option<String>,
    /// Whether we're downloading server.jar
    downloading_server: bool,
    /// Download progress (0.0 to 1.0)
    download_progress: f32,
    /// Download progress message
    download_status: String,
    /// Bytes downloaded
    bytes_downloaded: u64,
    /// Total bytes to download
    total_bytes: u64,
    /// Channel receiver for download progress updates
    download_receiver: Option<mpsc::Receiver<DownloadProgress>>,
}

/// Download progress update from background thread
struct DownloadProgress {
    progress: f32,      // 0.0 to 1.0
    status: String,     // Status message
    bytes_downloaded: u64,
    total_bytes: u64,
}

impl HomeTab {
    pub fn new() -> Self {
        Self {
            checking: false,
            java_found: false,
            java_version: String::new(),
            client_built: false,
            client_binary_path: None,
            resources_ready: false,
            server_jar_path: None,
            server_jar_error: None,
            downloading_server: false,
            download_progress: 0.0,
            download_status: String::new(),
            bytes_downloaded: 0,
            total_bytes: 0,
            download_receiver: None,
        }
    }

    /// Check if requirements are met
    fn check_requirements(&mut self, reset_binary: &Option<PathBuf>, resources_dir: &Option<PathBuf>) {
        self.checking = true;

        // Check Java
        self.java_found = false;
        self.java_version.clear();
        if let Ok(output) = Command::new("java").arg("-version").output() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("25") || stderr.contains("24") || stderr.contains("23") || stderr.contains("22") || stderr.contains("21") {
                self.java_found = true;
                // Extract version
                for line in stderr.lines() {
                    if line.contains("version") {
                        self.java_version = line.trim().to_string();
                        break;
                    }
                }
            }
        }

        // Check client binary
        self.client_built = reset_binary.is_some();

        // Check resources
        self.resources_ready = resources_dir.is_some();

        self.checking = false;
    }

    /// Download server.jar from Mojang's official servers (version 26.3) with progress reporting
    fn download_server_jar(&mut self, config: &crate::gui::app::LauncherConfig) {
        self.downloading_server = true;
        self.download_progress = 0.0;
        self.download_status = "Obtendo manifesto de versão...".to_string();
        self.bytes_downloaded = 0;
        self.total_bytes = 0;
        self.server_jar_error = None;

        // Create channel for progress updates
        let (tx, rx) = mpsc::channel();
        self.download_receiver = Some(rx);

        // Mojang version manifest URL
        let version_manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

        // Run download in a separate thread to not block UI
        let config_clone = config.clone();
        std::thread::spawn(move || {
            let client = reqwest::blocking::Client::new();

            // Helper to send progress updates
            let send_progress = |tx: &mpsc::Sender<DownloadProgress>, progress: f32, status: &str, bytes: u64, total: u64| {
                let _ = tx.send(DownloadProgress {
                    progress,
                    status: status.to_string(),
                    bytes_downloaded: bytes,
                    total_bytes: total,
                });
            };

            let download_result: Result<String, String> = (|| {
                // Step 1: Get version manifest to find the 26.3 version URL
                send_progress(&tx, 0.05, "Baixando manifesto de versão...", 0, 0);
                let response = client.get(version_manifest_url).send()
                    .map_err(|e| format!("Falha ao baixar manifesto de versão: {}", e))?;

                let manifest: serde_json::Value = response.json()
                    .map_err(|e| format!("Falha ao parsear manifesto de versão: {}", e))?;

                // Find version 26.3
                send_progress(&tx, 0.1, "Procurando versão 26.3...", 0, 0);
                let versions = manifest.get("versions").and_then(|v| v.as_array())
                    .ok_or("Campo 'versions' não encontrado no manifesto".to_string())?;

                let version_url = versions.iter()
                    .find(|v| v.get("id").and_then(|v| v.as_str()) == Some("26.3"))
                    .and_then(|v| v.get("url").and_then(|v| v.as_str()))
                    .ok_or("Versão 26.3 não encontrada no manifesto".to_string())?
                    .to_string();

                // Step 2: Get the version-specific manifest to find server.jar URL
                send_progress(&tx, 0.2, "Baixando manifesto da versão...", 0, 0);
                let version_response = client.get(&version_url).send()
                    .map_err(|e| format!("Falha ao baixar manifesto da versão: {}", e))?;

                let version_manifest: serde_json::Value = version_response.json()
                    .map_err(|e| format!("Falha ao parsear manifesto da versão: {}", e))?;

                let server_download_url = version_manifest.get("downloads")
                    .and_then(|d| d.get("server"))
                    .and_then(|s| s.get("url"))
                    .and_then(|u| u.as_str())
                    .ok_or("URL do server.jar não encontrada no manifesto".to_string())?
                    .to_string();

                // Step 3: Download the server.jar with progress
                send_progress(&tx, 0.3, "Iniciando download do server.jar...", 0, 0);
                let jar_response = client.get(&server_download_url).send()
                    .map_err(|e| format!("Falha ao baixar server.jar: {}", e))?;

                // Get content length for progress tracking
                let total_size = jar_response.content_length().unwrap_or(0);
                let mut downloaded: u64 = 0;
                let mut bytes_buffer = Vec::new();

                // Read in chunks to track progress
                let mut stream = jar_response;
                let mut buffer = [0u8; 8192];
                use std::io::Read;
                loop {
                    let n = stream.read(&mut buffer)
                        .map_err(|e| format!("Falha ao ler stream do server.jar: {}", e))?;
                    if n == 0 {
                        break;
                    }
                    bytes_buffer.extend_from_slice(&buffer[..n]);
                    downloaded += n as u64;

                    // Send progress update (0.3 to 0.9 range for download)
                    if total_size > 0 {
                        let progress = 0.3 + 0.6 * (downloaded as f32 / total_size as f32);
                        send_progress(&tx, progress, "Baixando server.jar...", downloaded, total_size);
                    } else {
                        send_progress(&tx, 0.5, "Baixando server.jar...", downloaded, 0);
                    }
                }

                // Step 4: Save to the reset-prepare directory
                send_progress(&tx, 0.9, "Salvando server.jar...", downloaded, total_size);
                let reset_path = config_clone.reset_path.as_ref()
                    .ok_or("Caminho do Reset não configurado".to_string())?;

                let server_jar_path = reset_path.join("server-26.3.jar");
                std::fs::write(&server_jar_path, &bytes_buffer)
                    .map_err(|e| format!("Falha ao salvar server.jar: {}", e))?;

                send_progress(&tx, 1.0, "Download concluído!", downloaded, total_size);
                Ok(server_jar_path.to_string_lossy().to_string())
            })();

            // Send final result
            match download_result {
                Ok(_path) => {
                    let _ = tx.send(DownloadProgress {
                        progress: 1.0,
                        status: format!("Server.jar baixado com sucesso!"),
                        bytes_downloaded: 0,
                        total_bytes: 0,
                    });
                }
                Err(e) => {
                    let _ = tx.send(DownloadProgress {
                        progress: -1.0, // Negative indicates error
                        status: format!("Erro: {}", e),
                        bytes_downloaded: 0,
                        total_bytes: 0,
                    });
                }
            }
        });
    }

    /// Prepare resources using reset-prepare
    fn prepare_resources(&mut self, config: &crate::gui::app::LauncherConfig, server_jar: &std::path::Path) {
        if let Some(reset_path) = &config.reset_path {
            let prepare_path = reset_path.join("crates").join("reset-prepare");
            if prepare_path.exists() {
                let status = Command::new("cargo")
                    .args([
                        "run", "--release", "-p", "reset-prepare", "--",
                        "--prepare", "--server-jar", server_jar.to_str().unwrap()
                    ])
                    .current_dir(reset_path)
                    .status();
                if status.map(|s| s.success()).unwrap_or(false) {
                    self.resources_ready = true;
                } else {
                    self.server_jar_error = Some("Falha ao preparar recursos. Verifique se o server.jar é válido (versão 26.3).".to_string());
                }
            } else {
                self.server_jar_error = Some("reset-prepare não encontrado. Verifique o caminho do Reset nas configurações.".to_string());
            }
        } else {
            self.server_jar_error = Some("Caminho do Reset não configurado. Configure nas Configurações.".to_string());
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
        horizon_binary: &Option<PathBuf>,
        resources_dir: &Option<PathBuf>,
        resolved_game_dir: &Option<PathBuf>,
        launch_status: &mut LaunchStatus,
        launch_error: &mut Option<String>,
        launch_game: impl FnOnce(),
    ) {
        // Requirements section
        section_header(ui, self.tr(config, "System Requirements", "Requisitos do Sistema"));

        // Check requirements button
        ui.horizontal(|ui| {
            if ui.add_enabled(!self.checking, egui::Button::new(
                egui::RichText::new(self.tr(config, "Check Requirements", "Verificar Requisitos")).size(13.0).color(WHITE)
            ).fill(egui::Color32::from_rgb(0x3A, 0x3A, 0x3A))
            .stroke(egui::Stroke::new(1.0_f32, SEPARATOR_COLOR))
            .rounding(egui::Rounding::same(6.0))
            .min_size(egui::Vec2::new(180.0, 36.0))).clicked() {
                self.check_requirements(horizon_binary, resources_dir);
            }

            if self.checking {
                ui.add_space(10.0);
                ui.spinner();
            }
        });
        ui.add_space(12.0);

        // Java status
        ui.horizontal(|ui| {
            let color = if self.java_found { egui::Color32::from_rgb(0x45, 0xC2, 0x7A) } else { BRAND_RED };
            ui.label(egui::RichText::new("●").size(14.0).color(color));
            ui.label(egui::RichText::new(self.tr(config, "Java JDK 25+", "Java JDK 25+")).size(13.0).color(WHITE));
            if self.java_found && !self.java_version.is_empty() {
                ui.add_space(10.0);
                ui.label(egui::RichText::new(&self.java_version).size(11.0).color(ICON_COLOR));
            }
        });
        ui.add_space(8.0);

        // Reset client status
        ui.horizontal(|ui| {
            let color = if self.client_built { egui::Color32::from_rgb(0x45, 0xC2, 0x7A) } else { BRAND_RED };
            ui.label(egui::RichText::new("●").size(14.0).color(color));
            ui.label(egui::RichText::new(self.tr(config, "Reset Client (built)", "Reset Client (construído)")).size(13.0).color(WHITE));
            if !self.client_built {
                ui.add_space(10.0);
                if secondary_button(ui, self.tr(config, "Build Client", "Construir Cliente"), true) {
                    // Run build command synchronously and wait for completion
                    if let Some(reset_path) = &config.reset_path {
                        let client_path = reset_path.join("crates").join("reset-client");
                        if client_path.exists() {
                            let status = Command::new("cargo")
                                .args(["build", "--release", "-p", "reset-client"])
                                .current_dir(reset_path)
                                .status();
                            if status.map(|s| s.success()).unwrap_or(false) {
                                // Check if binary was created
                                let binary_path = reset_path.join("target").join("release").join("reset-client");
                                #[cfg(windows)]
                                let binary_path = binary_path.with_extension("exe");
                                if binary_path.exists() {
                                    self.client_built = true;
                                    self.client_binary_path = Some(binary_path);
                                } else {
                                    self.server_jar_error = Some("Build succeeded but binary not found at expected location.".to_string());
                                }
                            } else {
                                self.server_jar_error = Some(self.tr(config, "Failed to build Reset client. Check if Java 25+ is installed.", "Falha ao construir o cliente Reset. Verifique se o Java 25+ está instalado.").to_string());
                            }
                        }
                    }
                }
            }
        });
        ui.add_space(8.0);

        // Resources status
        ui.horizontal(|ui| {
            let color = if self.resources_ready { egui::Color32::from_rgb(0x45, 0xC2, 0x7A) } else { BRAND_RED };
            ui.label(egui::RichText::new("●").size(14.0).color(color));
            ui.label(egui::RichText::new(self.tr(config, "Game Resources (prepared)", "Recursos do Jogo (preparados)")).size(13.0).color(WHITE));
        });
        ui.add_space(20.0);

        // Server.jar download section
        section_header(ui, self.tr(config, "Minecraft Server.jar", "Server.jar do Minecraft"));

        ui.label(egui::RichText::new(self.tr(config, "Download the Minecraft Java Edition server.jar (version 26.3) automatically from Mojang's official servers to prepare game resources.", "Baixe o server.jar do Minecraft Java Edition (versão 26.3) automaticamente dos servidores oficiais da Mojang para preparar os recursos do jogo.")).size(12.0).color(ICON_COLOR));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            // Download button for automatic server.jar download from Mojang
            if secondary_button(ui, self.tr(config, "Download server.jar 26.3", "Baixar server.jar 26.3"), !self.downloading_server && config.reset_path.is_some()) {
                self.download_server_jar(config);
            }
            ui.add_space(10.0);
            if let Some(path) = &self.server_jar_path {
                ui.label(egui::RichText::new(path.display().to_string()).size(11.0).color(ICON_COLOR));
            }
        });

        // Poll for download progress updates from background thread
        if self.downloading_server {
            // Take the receiver to avoid borrow checker issues
            if let Some(rx) = self.download_receiver.take() {
                let mut still_downloading = true;
                while let Ok(progress) = rx.try_recv() {
                    self.download_progress = progress.progress;
                    self.download_status = progress.status.clone();
                    self.bytes_downloaded = progress.bytes_downloaded;
                    self.total_bytes = progress.total_bytes;

                    // Check if download completed or failed
                    if progress.progress >= 1.0 {
                        self.downloading_server = false;
                        still_downloading = false;
                        // If successful, set the server_jar_path
                        if progress.progress > 0.0 {
                            if let Some(reset_path) = &config.reset_path {
                                let jar_path = reset_path.join("server-26.3.jar");
                                if jar_path.exists() {
                                    self.server_jar_path = Some(jar_path);
                                }
                            }
                        }
                    } else if progress.progress < 0.0 {
                        // Error occurred
                        self.downloading_server = false;
                        still_downloading = false;
                        self.server_jar_error = Some(progress.status);
                    }
                }
                // Put the receiver back if still downloading
                if still_downloading {
                    self.download_receiver = Some(rx);
                }
            }
        }

        // Download progress with progress bar
        if self.downloading_server || self.download_progress > 0.0 {
            ui.add_space(6.0);

            // Progress bar
            let progress_percent = (self.download_progress * 100.0) as u32;
            ui.add(egui::ProgressBar::new(self.download_progress)
                .text(format!("{}%", progress_percent))
                .animate(self.downloading_server)
                .desired_width(ui.available_width()));

            // Status text with bytes
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if self.downloading_server {
                    ui.spinner();
                    ui.add_space(6.0);
                }
                let status_text = if self.total_bytes > 0 {
                    format!("{} - {:.1} MB / {:.1} MB",
                        self.download_status,
                        self.bytes_downloaded as f64 / 1_048_576.0,
                        self.total_bytes as f64 / 1_048_576.0)
                } else {
                    self.download_status.clone()
                };
                ui.label(egui::RichText::new(&status_text).size(11.0).color(ICON_COLOR));
            });
        }

        if let Some(error) = &self.server_jar_error {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(error).size(11.0).color(BRAND_RED));
        }

        // If server.jar downloaded, show prepare button
        if self.server_jar_path.is_some() && !self.resources_ready {
            ui.add_space(10.0);
            if primary_button(ui, self.tr(config, "Prepare Resources with server.jar", "Preparar Recursos com server.jar"), true) {
                if let Some(server_jar) = self.server_jar_path.clone() {
                    self.prepare_resources(config, &server_jar);
                }
            }
        }
        ui.add_space(20.0);

        // Game directory section
        section_header(ui, self.tr(config, "Game Directory", "Diretório do Jogo"));

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(self.tr(config, "Folder:", "Pasta:")).size(13.0).color(ICON_COLOR));
            if let Some(game_dir) = resolved_game_dir {
                ui.label(egui::RichText::new(game_dir.display().to_string()).size(12.0).color(WHITE));
            } else {
                ui.label(egui::RichText::new(self.tr(config, "Not set", "Não definido")).size(12.0).color(BRAND_RED));
            }
            ui.add_space(10.0);
            if secondary_button(ui, self.tr(config, "Open Folder", "Abrir Pasta"), resolved_game_dir.is_some()) {
                if let Some(game_dir) = resolved_game_dir {
                    let _ = Command::new("explorer").arg(game_dir).spawn();
                }
            }
            if secondary_button(ui, self.tr(config, "Change", "Alterar"), true) {
                if let Some(path) = FileDialog::new().set_title(self.tr(config, "Select game directory", "Selecionar diretório do jogo")).pick_folder() {
                    config.game_dir = Some(path);
                }
            }
        });
        ui.add_space(20.0);

        // Play button section
        section_header(ui, self.tr(config, "Play", "Jogar"));

        // Check if ready to play
        let can_play = self.client_built && self.resources_ready && resolved_game_dir.is_some();

        ui.horizontal(|ui| {
            if primary_button(ui, self.tr(config, "PLAY", "JOGAR"), can_play) {
                launch_game();
            }

            if !can_play {
                ui.add_space(10.0);
                let mut missing = Vec::new();
                if !self.client_built { missing.push(self.tr(config, "Client not built", "Cliente não construído")); }
                if !self.resources_ready { missing.push(self.tr(config, "Resources not prepared", "Recursos não preparados")); }
                if resolved_game_dir.is_none() { missing.push(self.tr(config, "Game directory not set", "Diretório do jogo não definido")); }
                ui.label(egui::RichText::new(format!("{}: {}", self.tr(config, "Missing", "Requisitos"), missing.join(", "))).size(11.0).color(BRAND_RED));
            }
        });

        // Launch status
        ui.add_space(12.0);
        match *launch_status {
            LaunchStatus::Ready => {
                ui.label(egui::RichText::new(self.tr(config, "Ready to play", "Pronto para jogar")).size(12.0).color(ICON_COLOR));
            }
            LaunchStatus::Checking => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(self.tr(config, "Checking...", "Verificando...")).size(12.0).color(ICON_COLOR));
                });
            }
            LaunchStatus::Launching => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(self.tr(config, "Launching...", "Iniciando...")).size(12.0).color(BRAND_RED));
                });
            }
            LaunchStatus::Success => {
                ui.label(egui::RichText::new(self.tr(config, "Game launched successfully!", "Jogo iniciado com sucesso!")).size(12.0).color(egui::Color32::from_rgb(0x45, 0xC2, 0x7A)));
            }
            LaunchStatus::Error => {
                if let Some(error) = launch_error {
                    ui.label(egui::RichText::new(format!("{}: {}", self.tr(config, "Error", "Erro"), error)).size(12.0).color(BRAND_RED));
                }
            }
        }
    }
}

impl Default for HomeTab {
    fn default() -> Self {
        Self::new()
    }
}