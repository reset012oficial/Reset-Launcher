use std::path::{Path, PathBuf};
use std::process::Command;
use directories_next::ProjectDirs;
use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::gui::tabs::{HomeTab, SkinTab, AddTab, SettingsTab};
use crate::gui::theme::{
    theme, TITLEBAR_BG, SIDEBAR_BG, MAIN_BG, ICON_COLOR, SEPARATOR_COLOR,
    WHITE, BRAND_RED, TITLEBAR_HEIGHT, SIDEBAR_WIDTH, MAIN_CORNER_RADIUS,
};

/// Application state that persists between sessions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LauncherConfig {
    /// Player name (offline mode)
    pub player_name: String,

    /// Game directory path
    pub game_dir: Option<PathBuf>,

    /// Launch settings
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub render_distance: u32,
    pub memory_mb: u32,
    pub world_name: String,
    pub seed: Option<i64>,
    /// Seed as string for UI input
    pub seed_string: String,
    pub creative: bool,
    pub server_address: String,
    pub headless: bool,

    /// Skin path (custom .png skin)
    pub skin_path: Option<PathBuf>,

    /// Reset path (internal, not shown in UI - always uses launcher folder)
    #[serde(skip)]
    pub reset_path: Option<PathBuf>,

    /// Window maximized state
    pub maximized: bool,

    /// Language (en or pt)
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "pt".to_string()
}

impl LauncherConfig {
    pub fn default_with_reset_path(reset_path: PathBuf) -> Self {
        Self {
            player_name: String::new(),
            game_dir: None,
            window_width: 1280,
            window_height: 720,
            fullscreen: false,
            render_distance: 12,
            memory_mb: 4096,
            world_name: String::new(),
            seed: None,
            seed_string: String::new(),
            creative: false,
            server_address: String::new(),
            headless: false,
            skin_path: None,
            reset_path: Some(reset_path),
            maximized: false,
            language: "pt".to_string(),
        }
    }
}

/// Current sidebar selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidebarTab {
    Home,      // House icon - shows requirements, server.jar selection, play button
    Skin,      // A icon - change skin via .png file
    Add,       // + icon - install mods, textures, shaders, set username
    Settings,  // Gear icon - instance settings, memory, window size, fullscreen, credits
}

impl Default for SidebarTab {
    fn default() -> Self {
        SidebarTab::Home
    }
}

/// Launch status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchStatus {
    Ready,
    Checking,
    Launching,
    Success,
    Error,
}

/// Window control button types
#[derive(Clone, Copy)]
enum WindowControlType {
    Minimize,
    Maximize,
    Close,
}

/// Main application struct
pub struct ResetLauncherApp {
    /// Configuration
    config: LauncherConfig,
    /// Current sidebar tab
    current_tab: SidebarTab,
    /// Launch status
    launch_status: LaunchStatus,
    /// Launch error message
    launch_error: Option<String>,
    /// Reset client binary path
    reset_binary: Option<PathBuf>,
    /// Resources directory path
    resources_dir: Option<PathBuf>,
    /// Game directory path (resolved)
    resolved_game_dir: Option<PathBuf>,
    /// Tab instances
    home_tab: HomeTab,
    skin_tab: SkinTab,
    add_tab: AddTab,
    settings_tab: SettingsTab,
    /// Config file path
    config_path: PathBuf,
    /// Window maximized state
    is_maximized: bool,
    /// Flag to request game launch (avoids borrow issues with closure)
    request_launch: bool,
}

impl ResetLauncherApp {
    /// Create a new application instance
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load config from disk
        let config_path = Self::config_file_path();
        let config = Self::load_config(&config_path);

        // Determine reset path
        let reset_path = config.reset_path.clone().unwrap_or_else(|| {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));
            // Binary is at horizon-offline-launcher/target/release/...
            // reset-client is sibling of horizon-offline-launcher, so go up 3 levels
            exe_dir.join("..").join("..").join("..").join("reset-client")
        });

        // Resolve paths
        let (reset_binary, resources_dir, resolved_game_dir) = Self::resolve_paths(&reset_path, &config);

        // Create tab instances
        let home_tab = HomeTab::new();
        let skin_tab = SkinTab::new();
        let add_tab = AddTab::new();
        let settings_tab = SettingsTab::new();

        // Apply theme
        cc.egui_ctx.set_visuals(theme());

        // Set maximized state from config
        let is_maximized = config.maximized;

        Self {
            config,
            current_tab: SidebarTab::Home,
            launch_status: LaunchStatus::Ready,
            launch_error: None,
            reset_binary,
            resources_dir,
            resolved_game_dir,
            home_tab,
            skin_tab,
            add_tab,
            settings_tab,
            config_path,
            is_maximized,
            request_launch: false,
        }
    }

    /// Get the config file path
    fn config_file_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "reset", "reset-launcher") {
            let mut path = proj_dirs.config_dir().to_path_buf();
            std::fs::create_dir_all(&path).ok();
            path.push("config.json");
            path
        } else {
            PathBuf::from("reset-launcher-config.json")
        }
    }

    /// Load config from file
    fn load_config(path: &Path) -> LauncherConfig {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        LauncherConfig::default()
    }

    /// Save config to file
    fn save_config(&self) {
        if let Ok(content) = serde_json::to_string_pretty(&self.config) {
            let _ = std::fs::write(&self.config_path, content);
        }
    }

    /// Resolve paths based on reset path and config
    fn resolve_paths(reset_path: &Path, config: &LauncherConfig) -> (Option<PathBuf>, Option<PathBuf>, Option<PathBuf>) {
        // Reset client binary
        let binary = reset_path.join("target").join("release").join("reset-client");
        #[cfg(windows)]
        let binary = binary.with_extension("exe");

        let reset_binary = if binary.exists() { Some(binary) } else { None };

        // Resources directory
        let resources = reset_path.join(".local").join("horizon-client").join("26.3");
        let resources_dir = if resources.join("horizon-prepare.json").exists() {
            Some(resources)
        } else {
            None
        };

        // Game directory
        let game_dir = config.game_dir.clone().or_else(|| {
            // Use standard directories matching README.md:
            // Windows: %APPDATA%\reset-offline\
            // Linux: ~/.local/share/reset-offline/
            // macOS: ~/Library/Application Support/reset-offline/
            if let Some(proj_dirs) = ProjectDirs::from("", "reset", "reset-offline") {
                Some(proj_dirs.data_dir().to_path_buf())
            } else {
                Some(PathBuf::from(".").join("reset-offline"))
            }
        });

        let resolved_game_dir = game_dir.map(|p| {
            std::fs::create_dir_all(&p).ok();
            p
        });

        (reset_binary, resources_dir, resolved_game_dir)
    }

    /// Refresh paths after reset path change
    fn refresh_paths(&mut self) {
        if let Some(reset_path) = &self.config.reset_path {
            let (binary, resources, game_dir) = Self::resolve_paths(reset_path, &self.config);
            self.reset_binary = binary;
            self.resources_dir = resources;
            self.resolved_game_dir = game_dir;
        }
    }

    /// Launch the Reset client
    fn launch_game(&mut self) {
        // Refresh paths to pick up any newly built binary
        self.refresh_paths();

        self.launch_status = LaunchStatus::Launching;
        self.launch_error = None;

        // Validate binary exists
        let binary = match &self.reset_binary {
            Some(b) => b.clone(),
            None => {
                self.launch_status = LaunchStatus::Error;
                self.launch_error = Some("Reset client binary not found. Please build the client first.".to_string());
                return;
            }
        };

        // Validate resources
        if self.resources_dir.is_none() {
            self.launch_status = LaunchStatus::Error;
            self.launch_error = Some("Resources not found. Please run 'prepare' with a server.jar first.".to_string());
            return;
        }

        let resources = self.resources_dir.as_ref().unwrap().clone();
        let game_dir = self.resolved_game_dir.clone().unwrap_or_else(|| PathBuf::from("."));

        // Build arguments
        let mut args = vec![
            "--resources".to_string(), resources.to_string_lossy().to_string(),
            "--render-distance".to_string(), self.config.render_distance.to_string(),
            "--width".to_string(), self.config.window_width.to_string(),
            "--height".to_string(), self.config.window_height.to_string(),
            "--game-dir".to_string(), game_dir.to_string_lossy().to_string(),
        ];

        if self.config.fullscreen {
            args.push("--fullscreen".to_string());
        }

        if self.config.headless {
            args.push("--headless".to_string());
        }

        if !self.config.player_name.is_empty() {
            args.push("--name".to_string());
            args.push(self.config.player_name.clone());
        }

        if !self.config.world_name.is_empty() {
            args.push("--world".to_string());
            args.push(self.config.world_name.clone());
        }

        if let Some(seed) = self.config.seed {
            args.push("--seed".to_string());
            args.push(seed.to_string());
        }

        if self.config.creative {
            args.push("--creative".to_string());
        }

        if !self.config.server_address.is_empty() {
            args.push("--server".to_string());
            args.push(self.config.server_address.clone());
        }

        // Launch the process
        let mut cmd = Command::new(&binary);
        cmd.args(&args)
            .current_dir(&game_dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        match cmd.spawn() {
            Ok(_child) => {
                self.launch_status = LaunchStatus::Success;
                self.save_config();
            }
            Err(e) => {
                self.launch_status = LaunchStatus::Error;
                self.launch_error = Some(format!("Failed to launch: {}", e));
            }
        }
    }

    /// Draw a 4-point geometric star on the given painter
    fn draw_star(painter: &egui::Painter, center: egui::Pos2, radius: f32, color: egui::Color32) {
        let mut points = Vec::with_capacity(8);
        // 4-point star: alternate between outer and inner radius
        // Inner radius 0.6 makes a more balanced 4-pointed star
        for i in 0..8 {
            let angle = (i as f32) * std::f32::consts::PI / 4.0 - std::f32::consts::PI / 2.0;
            let r = if i % 2 == 0 { radius } else { radius * 0.6 };
            points.push(egui::pos2(
                center.x + angle.cos() * r,
                center.y + angle.sin() * r,
            ));
        }
        painter.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
    }

    /// Draw house icon on the given painter
    fn draw_house_icon(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
        let s = size / 2.0;
        // Simple house shape: triangle roof + rectangle base
        let roof_points = vec![
            egui::pos2(center.x, center.y - s),           // top
            egui::pos2(center.x - s * 0.9, center.y),     // bottom left of roof
            egui::pos2(center.x + s * 0.9, center.y),     // bottom right of roof
        ];
        painter.add(egui::Shape::convex_polygon(roof_points, color, egui::Stroke::NONE));

        // House base
        let base_rect = egui::Rect::from_min_max(
            egui::pos2(center.x - s * 0.6, center.y),
            egui::pos2(center.x + s * 0.6, center.y + s * 0.8),
        );
        painter.rect_filled(base_rect, 0.0, color);

        // Door
        let door_rect = egui::Rect::from_min_max(
            egui::pos2(center.x - s * 0.15, center.y + s * 0.2),
            egui::pos2(center.x + s * 0.15, center.y + s * 0.8),
        );
        painter.rect_filled(door_rect, 0.0, SIDEBAR_BG);
    }

    /// Draw A icon (circle with stylized A) on the given painter
    fn draw_a_icon(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
        // Circle outline
        painter.circle_stroke(center, size / 2.0, egui::Stroke::new(2.5_f32, color));

        // Stylized A inside
        let s = size * 0.35;
        // Left diagonal
        painter.line_segment([
            egui::pos2(center.x - s, center.y + s),
            egui::pos2(center.x, center.y - s),
        ], egui::Stroke::new(3.0_f32, color));
        // Right diagonal
        painter.line_segment([
            egui::pos2(center.x, center.y - s),
            egui::pos2(center.x + s, center.y + s),
        ], egui::Stroke::new(3.0_f32, color));
        // Crossbar
        painter.line_segment([
            egui::pos2(center.x - s * 0.5, center.y),
            egui::pos2(center.x + s * 0.5, center.y),
        ], egui::Stroke::new(3.0_f32, color));
    }

    /// Draw plus icon (+) on the given painter
    fn draw_plus_icon(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
        let thickness = 5.0;
        let half = size / 2.0;

        // Vertical bar
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(center.x - thickness / 2.0, center.y - half),
                egui::pos2(center.x + thickness / 2.0, center.y + half),
            ),
            thickness,
            color,
        );

        // Horizontal bar
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(center.x - half, center.y - thickness / 2.0),
                egui::pos2(center.x + half, center.y + thickness / 2.0),
            ),
            thickness,
            color,
        );
    }

    /// Draw gear/settings icon (hexagon with center circle) on the given painter
    fn draw_gear_icon(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
        let r_outer = size / 2.0;
        let r_inner = r_outer * 0.6;

        // Draw hexagon (6 points)
        let mut hex_points = Vec::with_capacity(6);
        for i in 0..6 {
            let angle = (i as f32) * std::f32::consts::PI * 2.0 / 6.0 - std::f32::consts::PI / 6.0;
            hex_points.push(egui::pos2(
                center.x + angle.cos() * r_outer,
                center.y + angle.sin() * r_outer,
            ));
        }
        // Draw as stroke (outline)
        for i in 0..6 {
            let p1 = hex_points[i];
            let p2 = hex_points[(i + 1) % 6];
            painter.line_segment([p1, p2], egui::Stroke::new(2.5_f32, color));
        }

        // Center circle
        painter.circle_stroke(center, r_inner, egui::Stroke::new(2.5_f32, color));
    }
}

impl eframe::App for ResetLauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let screen_rect = ctx.screen_rect();
        let available_width = screen_rect.width() - SIDEBAR_WIDTH;
        let available_height = screen_rect.height() - TITLEBAR_HEIGHT;

        // ============================================================
        // TOP BAR (Titlebar) - 44px height, custom drawn
        // ============================================================
        egui::TopBottomPanel::top("titlebar")
            .exact_height(TITLEBAR_HEIGHT)
            .frame(egui::Frame::none().fill(TITLEBAR_BG))
            .show(ctx, |ui| {
                let painter = ui.painter();

                // Draw star logo (4-point geometric star) - approximately 27x27 at position (7, 7)
                let star_center = egui::pos2(20.5, TITLEBAR_HEIGHT / 2.0);
                Self::draw_star(painter, star_center, 13.5, BRAND_RED);

                // Draw "Reset Launcher" text - positioned ~12px from star
                let text_pos = egui::pos2(46.0, TITLEBAR_HEIGHT / 2.0);
                painter.text(
                    text_pos,
                    egui::Align2::LEFT_CENTER,
                    "Reset Launcher",
                    egui::FontId::new(23.0, egui::FontFamily::Proportional),
                    WHITE,
                );

                // Draw window controls on the right
                // Each control: 58px wide, 44px tall
                let control_width = 58.0;
                let controls_start_x = screen_rect.width() - 3.0 * control_width;

                // Minimize button (leftmost of the three)
                let minimize_rect = egui::Rect::from_min_size(
                    egui::pos2(controls_start_x, 0.0),
                    egui::vec2(control_width, TITLEBAR_HEIGHT),
                );
                self.draw_window_control(ui, minimize_rect, WindowControlType::Minimize);

                // Maximize/Restore button
                let maximize_rect = egui::Rect::from_min_size(
                    egui::pos2(controls_start_x + control_width, 0.0),
                    egui::vec2(control_width, TITLEBAR_HEIGHT),
                );
                self.draw_window_control(ui, maximize_rect, WindowControlType::Maximize);

                // Close button (rightmost)
                let close_rect = egui::Rect::from_min_size(
                    egui::pos2(controls_start_x + 2.0 * control_width, 0.0),
                    egui::vec2(control_width, TITLEBAR_HEIGHT),
                );
                self.draw_window_control(ui, close_rect, WindowControlType::Close);

                // Handle drag to move window (titlebar area except controls)
                let drag_rect = egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(controls_start_x, TITLEBAR_HEIGHT),
                );
                // Use an invisible button for drag handling
                let drag_response = ui.allocate_rect(drag_rect, egui::Sense::drag());
                if drag_response.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
            });

        // ============================================================
        // SIDEBAR - 45px width, custom drawn
        // ============================================================
        egui::SidePanel::left("sidebar")
            .exact_width(SIDEBAR_WIDTH)
            .frame(egui::Frame::none().fill(SIDEBAR_BG))
            .show(ctx, |ui| {
                let icon_x = SIDEBAR_WIDTH / 2.0;
                let item_height = 57.0; // Approximate height for first three items

                // Item 1: HOME (House icon) - positioned ~15px from top of sidebar
                let home_y = TITLEBAR_HEIGHT + 15.0 + item_height / 2.0;
                let home_center = egui::pos2(icon_x, home_y);
                let home_rect = egui::Rect::from_center_size(home_center, egui::vec2(SIDEBAR_WIDTH, item_height));

                let home_response = ui.allocate_rect(home_rect, egui::Sense::click());
                if home_response.hovered() {
                    ui.painter().rect_filled(home_rect, 0.0, egui::Color32::from_rgb(0x3A, 0x3A, 0x3A));
                    ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                let icon_color = if home_response.hovered() { WHITE } else { ICON_COLOR };
                Self::draw_house_icon(ui.painter(), home_center, 30.0, icon_color);
                if home_response.clicked() {
                    self.current_tab = SidebarTab::Home;
                }

                // Separator 1
                let sep1_y = TITLEBAR_HEIGHT + 15.0 + item_height;
                ui.painter().line_segment([
                    egui::pos2(0.0, sep1_y),
                    egui::pos2(SIDEBAR_WIDTH, sep1_y),
                ], egui::Stroke::new(1.0_f32, SEPARATOR_COLOR));

                // Item 2: SKIN (A icon) - centered in 56px height
                let skin_y = sep1_y + 56.0 / 2.0;
                let skin_center = egui::pos2(icon_x, skin_y);
                let skin_rect = egui::Rect::from_center_size(skin_center, egui::vec2(SIDEBAR_WIDTH, 56.0));

                let skin_response = ui.allocate_rect(skin_rect, egui::Sense::click());
                if skin_response.hovered() {
                    ui.painter().rect_filled(skin_rect, 0.0, egui::Color32::from_rgb(0x3A, 0x3A, 0x3A));
                    ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                let icon_color = if skin_response.hovered() { WHITE } else { ICON_COLOR };
                Self::draw_a_icon(ui.painter(), skin_center, 30.0, icon_color);
                if skin_response.clicked() {
                    self.current_tab = SidebarTab::Skin;
                }

                // Separator 2
                let sep2_y = sep1_y + 56.0;
                ui.painter().line_segment([
                    egui::pos2(0.0, sep2_y),
                    egui::pos2(SIDEBAR_WIDTH, sep2_y),
                ], egui::Stroke::new(1.0_f32, SEPARATOR_COLOR));

                // Item 3: ADD (+ icon) - centered in 56px height
                let add_y = sep2_y + 56.0 / 2.0;
                let add_center = egui::pos2(icon_x, add_y);
                let add_rect = egui::Rect::from_center_size(add_center, egui::vec2(SIDEBAR_WIDTH, 56.0));

                let add_response = ui.allocate_rect(add_rect, egui::Sense::click());
                if add_response.hovered() {
                    ui.painter().rect_filled(add_rect, 0.0, egui::Color32::from_rgb(0x3A, 0x3A, 0x3A));
                    ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                let icon_color = if add_response.hovered() { WHITE } else { ICON_COLOR };
                Self::draw_plus_icon(ui.painter(), add_center, 35.0, icon_color);
                if add_response.clicked() {
                    self.current_tab = SidebarTab::Add;
                }

                // Separator 3
                let sep3_y = sep2_y + 56.0;
                ui.painter().line_segment([
                    egui::pos2(0.0, sep3_y),
                    egui::pos2(SIDEBAR_WIDTH, sep3_y),
                ], egui::Stroke::new(1.0_f32, SEPARATOR_COLOR));

                // Item 4: SETTINGS (Gear icon) - anchored at bottom
                let settings_y = TITLEBAR_HEIGHT + available_height - 40.0; // ~40px from bottom
                let settings_center = egui::pos2(icon_x, settings_y);
                let settings_rect = egui::Rect::from_center_size(settings_center, egui::vec2(SIDEBAR_WIDTH, 56.0));

                let settings_response = ui.allocate_rect(settings_rect, egui::Sense::click());
                if settings_response.hovered() {
                    ui.painter().rect_filled(settings_rect, 0.0, egui::Color32::from_rgb(0x3A, 0x3A, 0x3A));
                    ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                let icon_color = if settings_response.hovered() { WHITE } else { ICON_COLOR };
                Self::draw_gear_icon(ui.painter(), settings_center, 32.0, icon_color);
                if settings_response.clicked() {
                    self.current_tab = SidebarTab::Settings;
                }
            });

        // ============================================================
        // MAIN CONTENT AREA - fills remaining space with rounded top corners
        // ============================================================
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(MAIN_BG).rounding(egui::Rounding {
                nw: MAIN_CORNER_RADIUS,
                ne: MAIN_CORNER_RADIUS,
                sw: 0.0,
                se: 0.0,
            }))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(available_width);
                        ui.add_space(20.0);

                        match self.current_tab {
                            SidebarTab::Home => {
                                self.home_tab.show(
                                    ui,
                                    &mut self.config,
                                    &self.reset_binary,
                                    &self.resources_dir,
                                    &self.resolved_game_dir,
                                    &mut self.launch_status,
                                    &mut self.launch_error,
                                    || self.request_launch = true,
                                );
                                // Check if launch was requested
                                if self.request_launch {
                                    self.request_launch = false;
                                    self.launch_game();
                                }
                            }
                            SidebarTab::Skin => {
                                self.skin_tab.show(
                                    ui,
                                    &mut self.config,
                                    &self.resolved_game_dir,
                                );
                            }
                            SidebarTab::Add => {
                                self.add_tab.show(
                                    ui,
                                    &mut self.config,
                                    &self.resolved_game_dir,
                                );
                            }
                            SidebarTab::Settings => {
                                self.settings_tab.show(
                                    ui,
                                    &mut self.config,
                                );
                            }
                        }

                        ui.add_space(20.0);
                    });
            });

        // Request repaint for smooth animations
        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "config", &self.config);
        eframe::set_value(storage, "current_tab", &self.current_tab);
        eframe::set_value(storage, "is_maximized", &self.is_maximized);
        self.save_config();
    }
}

impl ResetLauncherApp {
    /// Draw a window control button (minimize, maximize, close)
    fn draw_window_control(&mut self, ui: &mut egui::Ui, rect: egui::Rect, control_type: WindowControlType) {
        let response = ui.allocate_rect(rect, egui::Sense::click());
        let painter = ui.painter();

        // Draw hover background
        if response.hovered() {
            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(0x3A, 0x3A, 0x3A));
            ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }

        let center = rect.center();
        let icon_color = if response.hovered() { WHITE } else { ICON_COLOR };

        match control_type {
            WindowControlType::Minimize => {
                // Horizontal line: ~35x5 px
                let line_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(35.0, 5.0),
                );
                painter.rect_filled(line_rect, 2.0, icon_color);
            }
            WindowControlType::Maximize => {
                // Square outline: ~28x24 px with rounded corners
                let square_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(28.0, 24.0),
                );
                painter.rect_stroke(square_rect, 4.0, egui::Stroke::new(2.0_f32, icon_color));
            }
            WindowControlType::Close => {
                // X: two diagonal lines ~25px
                let size = 12.5;
                let p1 = egui::pos2(center.x - size, center.y - size);
                let p2 = egui::pos2(center.x + size, center.y + size);
                let p3 = egui::pos2(center.x + size, center.y - size);
                let p4 = egui::pos2(center.x - size, center.y + size);
                painter.line_segment([p1, p2], egui::Stroke::new(2.5_f32, icon_color));
                painter.line_segment([p3, p4], egui::Stroke::new(2.5_f32, icon_color));
            }
        }

        // Handle clicks
        if response.clicked() {
            match control_type {
                WindowControlType::Minimize => {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
                WindowControlType::Maximize => {
                    self.is_maximized = !self.is_maximized;
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(self.is_maximized));
                    self.config.maximized = self.is_maximized;
                    self.save_config();
                }
                WindowControlType::Close => {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
}