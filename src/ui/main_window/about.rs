use std::path::{Path, PathBuf};
use std::process::Command;

use eframe::egui;

use crate::capture;
use crate::clipboard;
use crate::config::{Config, LogLevel};
use crate::logging;
use crate::state::State;
use crate::ui::{ICON_PNG, WINDOW_TITLE, widgets};

const WINDOW_ID: &str = "colorpickle-about";
const OPENER: &str = "xdg-open";
const LOGO_SIZE: f32 = 144.0;
/// The gap above the logo, between the logo and the name, and below the name.
const LOGO_SPACING: f32 = 12.0;
/// Gaps inside the identity block below the name.
const LABEL_SPACING: f32 = 2.0;
const SECTION_SPACING: f32 = 14.0;
/// Tighter than `SECTION_SPACING` where a row belongs to the block above it.
const TIGHT_SPACING: f32 = 6.0;
const ROW_SPACING: f32 = 14.0;
const NAME_FONT_SIZE: f32 = 28.0;
const PATH_FONT_SIZE: f32 = 13.0;
const SECTION_TITLE_FONT_SIZE: f32 = 15.0;
const WINDOW_WIDTH: f32 = 580.0;
/// Both the opening size and the minimum: the collapsed content measures 529.
const WINDOW_HEIGHT: f32 = 550.0;
const LOG_ROW_LABEL: &str = "Logs";
const LOG_LEVEL_ROW_LABEL: &str = "Log level";
/// Title column of the path table; the value column starts after it.
const LABEL_COLUMN_WIDTH: f32 = 76.0;
const LINK_WEBSITE: &str = "Website";
const LINK_RELEASES: &str = "Releases";
const LINK_ISSUES: &str = "Report a bug";
/// Kept in step with the copyright in Cargo.toml.
const COPYRIGHT: &str = "© 2026 dcog989";
const SYSTEM_INFO_TITLE: &str = "System information";
const COPY_TOOLTIP: &str = "Copy environment info (Ctrl+C)";
const PATH_PADDING: f32 = 2.0;
/// egui bundles no bold font, so titles are overpainted by this much to gain weight.
const BOLD_SMUDGE: f32 = 0.5;

#[derive(Default)]
pub struct About {
    open: bool,
    logo: Option<egui::TextureHandle>,
    /// Gathered once per opening: it shells out to `plasmashell` and `lspci`.
    info: Option<String>,
}

/// What the panel changed this frame.
#[derive(Default)]
pub struct Outcome {
    pub log_level_changed: bool,
    pub toast: Option<String>,
}

impl About {
    pub fn open(&mut self) {
        self.open = true;
        self.info = None;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Returns what the panel changed: its log level, or a toast to show.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        frame: &eframe::Frame,
        config: &mut Config,
    ) -> Outcome {
        if !self.open {
            return Outcome::default();
        }
        if self.logo.is_none() {
            self.logo = logo_texture(ctx);
        }
        let first_frame = self.info.is_none();
        if first_frame {
            self.info = Some(environment_info(frame));
        }
        let info = self.info.as_deref().unwrap_or_default();

        let mut viewport = egui::ViewportBuilder::default()
            .with_title(format!("About {WINDOW_TITLE}"))
            .with_app_id(crate::ui::APP_ID)
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_min_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]);
        // The window is created on the first frame, so the icon only has to be
        // decoded and handed over then.
        if first_frame && let Some(icon) = crate::ui::app_icon() {
            viewport = viewport.with_icon(icon);
        }

        let mut open = self.open;
        let mut outcome = Outcome::default();
        // A viewport of its own: the diagnostics make the panel taller than the
        // main window, which would clip it.
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of(WINDOW_ID),
            viewport,
            |ctx, _class| {
                if ctx.input(|input| input.viewport().close_requested())
                    || ctx.input(|input| input.key_pressed(egui::Key::Escape))
                {
                    open = false;
                }
                let panel = egui::Frame::central_panel(ctx.style())
                    .inner_margin(egui::Margin::same(LOGO_SPACING as i8));
                egui::CentralPanel::default().frame(panel).show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            outcome = self.contents(ui, &mut config.log_level, info);
                        });
                });
                if copy_shortcut(ctx) {
                    outcome.toast = Some(copy_environment_info(info));
                }
            },
        );
        self.open = open;
        outcome
    }

    fn contents(&self, ui: &mut egui::Ui, level: &mut LogLevel, info: &str) -> Outcome {
        let mut outcome = Outcome::default();
        ui.vertical_centered(|ui| {
            // Every gap in the identity block is explicit, so the space above
            // and below the logo and the name is the same.
            ui.spacing_mut().item_spacing.y = 0.0;
            if let Some(logo) = &self.logo {
                ui.add(egui::Image::new(logo).fit_to_exact_size(egui::vec2(LOGO_SIZE, LOGO_SIZE)));
            }
            ui.add_space(LOGO_SPACING);
            ui.label(
                egui::RichText::new(WINDOW_TITLE)
                    .size(NAME_FONT_SIZE)
                    .strong(),
            );
            ui.add_space(LOGO_SPACING);
            version_row(ui);
            ui.add_space(LABEL_SPACING);
            ui.weak(format!("{} · {COPYRIGHT}", env!("CARGO_PKG_LICENSE")));
            ui.add_space(LABEL_SPACING);
            if let Some(message) = links_row(ui, info) {
                outcome.toast = Some(message);
            }
        });

        ui.add_space(TIGHT_SPACING);
        if system_information(ui, info) {
            outcome.toast = Some(copy_environment_info(info));
        }

        ui.add_space(TIGHT_SPACING);
        ui.separator();
        ui.add_space(SECTION_SPACING);

        for (index, (label, path)) in paths().into_iter().enumerate() {
            if index > 0 {
                ui.add_space(ROW_SPACING);
            }
            ui.horizontal(|ui| {
                label_column(ui, label);
                path_value(ui, path);
            });
            if label == LOG_ROW_LABEL {
                ui.add_space(ROW_SPACING);
                ui.horizontal(|ui| {
                    label_column(ui, LOG_LEVEL_ROW_LABEL);
                    outcome.log_level_changed = log_level_combo(ui, level);
                });
            }
        }
        outcome
    }
}

/// The title column: every value in the table starts at the same x.
fn label_column(ui: &mut egui::Ui, text: &str) {
    let font = egui::FontId::proportional(SECTION_TITLE_FONT_SIZE);
    let color = ui.visuals().strong_text_color();
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, color);
    let width = galley.size().x.max(LABEL_COLUMN_WIDTH);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(width, galley.size().y), egui::Sense::hover());
    let painter = ui.painter();
    painter.galley(rect.left_top(), galley.clone(), color);
    painter.galley(
        rect.left_top() + egui::vec2(BOLD_SMUDGE, 0.0),
        galley,
        color,
    );
}

/// Centred like the version, and each link names its destination on hover.
fn links_row(ui: &mut egui::Ui, info: &str) -> Option<String> {
    let repository = env!("CARGO_PKG_REPOSITORY");
    let links = [
        (LINK_WEBSITE, repository.to_owned()),
        (LINK_RELEASES, format!("{repository}/releases")),
        (LINK_ISSUES, format!("{repository}/issues/new")),
    ];
    let font = egui::TextStyle::Body.resolve(ui.style());
    let color = ui.visuals().hyperlink_color;
    let width: f32 = links
        .iter()
        .map(|(label, _)| {
            ui.painter()
                .layout_no_wrap((*label).to_owned(), font.clone(), color)
                .size()
                .x
        })
        .sum::<f32>()
        + ui.spacing().item_spacing.x * (links.len() - 1) as f32;

    let mut toast = None;
    ui.horizontal(|ui| {
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        for (label, url) in links {
            if ui.link(label).on_hover_text(url.as_str()).clicked() {
                if label == LINK_ISSUES {
                    toast = Some(copy_environment_info(info));
                }
                open_url(&url);
            }
        }
    });
    toast
}

fn version_row(ui: &mut egui::Ui) {
    let text = format!("Version {}", env!("CARGO_PKG_VERSION"));
    ui.horizontal(|ui| {
        let font = egui::TextStyle::Body.resolve(ui.style());
        let width = ui
            .painter()
            .layout_no_wrap(text.clone(), font, ui.visuals().text_color())
            .size()
            .x;
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        ui.label(text);
    });
}

/// Centred header, left-aligned content. The arrow and the title both toggle,
/// and the copy button rides beside the title.
fn system_information(ui: &mut egui::Ui, info: &str) -> bool {
    let id = ui.make_persistent_id(SYSTEM_INFO_TITLE);
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
    let mut copied = false;

    let font = egui::TextStyle::Body.resolve(ui.style());
    let title_width = ui
        .painter()
        .layout_no_wrap(
            SYSTEM_INFO_TITLE.to_owned(),
            font,
            ui.visuals().text_color(),
        )
        .size()
        .x;
    let row_width = ui.spacing().indent
        + title_width
        + widgets::COPY_BUTTON_SIZE
        + 2.0 * ui.spacing().item_spacing.x;

    let mut toggled = false;
    ui.horizontal(|ui| {
        ui.add_space(((ui.available_width() - row_width) / 2.0).max(0.0));
        let (_, arrow) = ui.allocate_exact_size(
            egui::vec2(ui.spacing().indent, ui.spacing().icon_width),
            egui::Sense::click(),
        );
        let arrow = arrow.on_hover_cursor(egui::CursorIcon::PointingHand);
        let openness = if state.is_open() { 1.0 } else { 0.0 };
        egui::collapsing_header::paint_default_icon(ui, openness, &arrow);
        let title = ui
            .add(egui::Label::new(SYSTEM_INFO_TITLE).sense(egui::Sense::click()))
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        copied = widgets::copy_button(ui, COPY_TOOLTIP);
        toggled = arrow.clicked() || title.clicked();
    });
    if toggled {
        state.toggle(ui);
    }
    if state.is_open() {
        ui.indent(id, |ui| {
            ui.monospace(info);
        });
    }
    state.store(ui.ctx());
    copied
}

/// Ctrl+C on the panel copies the same text as the button, so the action is
/// reachable without the mouse.
fn copy_shortcut(ctx: &egui::Context) -> bool {
    !ctx.egui_wants_keyboard_input()
        && ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::C))
}

/// The clipboard is served by this process, so hand it to the helper like every
/// other copy: the text outlives the window.
fn copy_environment_info(info: &str) -> String {
    match clipboard::set_text(info) {
        Ok(()) => "Copied environment info".to_owned(),
        Err(error) => format!("Copy failed: {error}"),
    }
}

fn environment_info(frame: &eframe::Frame) -> String {
    let mut lines = vec![format!("{WINDOW_TITLE} {}", env!("CARGO_PKG_VERSION"))];
    lines.push(format!("Build: {}", build_profile()));
    lines.push(format!("Install: {}", install_kind()));
    let os = read_os_name().unwrap_or_else(|| "unknown".to_owned());
    lines.push(format!("OS: {os} ({})", std::env::consts::ARCH));
    if let Some(kernel) = read_trimmed("/proc/sys/kernel/osrelease") {
        lines.push(format!("Kernel: {kernel}"));
    }
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").ok();
    let session = std::env::var("XDG_SESSION_TYPE").ok();
    if let Some(desktop) = desktop {
        let desktop = if desktop.eq_ignore_ascii_case("kde") {
            match plasma_version() {
                Some(version) => format!("{desktop} Plasma {version}"),
                None => desktop,
            }
        } else {
            desktop
        };
        match session {
            Some(session) => lines.push(format!("Desktop: {desktop} ({session})")),
            None => lines.push(format!("Desktop: {desktop}")),
        }
    } else if let Some(session) = session {
        lines.push(format!("Session: {session}"));
    }
    if let Some(cpu) = cpu_name() {
        lines.push(format!("CPU: {cpu}"));
    }
    if let Some(memory) = memory_gib() {
        lines.push(format!("Memory: {memory:.1} GiB"));
    }
    if let Some(graphics) = graphics(frame) {
        lines.push(format!("Graphics: {graphics}"));
    }
    let displays = displays(frame);
    if !displays.is_empty() {
        lines.push("Displays:".to_owned());
        lines.extend(displays);
    }
    let sources = capture::available_sources();
    let mut backends = sources.iter().map(|source| source.label());
    if let Some(preferred) = backends.next() {
        let fallbacks: Vec<&str> = backends.collect();
        lines.push(if fallbacks.is_empty() {
            format!("Capture: {preferred}")
        } else {
            format!("Capture: {preferred} (fallbacks: {})", fallbacks.join(", "))
        });
    }
    lines.join("\n")
}

fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

/// An AppImage runs from a temp mount and is denied by KWin's ScreenShot2
/// allowlist, so this explains the most common capture report.
/// shortcut: only the AppImage variable and the /usr prefix are checked; a
/// package installed elsewhere reports as a local build.
fn install_kind() -> &'static str {
    if std::env::var_os("APPIMAGE").is_some() {
        "AppImage"
    } else if std::env::current_exe().is_ok_and(|path| path.starts_with("/usr/")) {
        "system package"
    } else {
        "local build"
    }
}

/// The adapter names the card and the rendering backend, sysfs carries the
/// driver and PCI id. lspci's board name is not used: pci.ids reuses one entry
/// for several SKUs and named a 7900 XT as a 7900 XTX.
fn graphics(frame: &eframe::Frame) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if let Some(state) = frame.wgpu_render_state() {
        let info = state.adapter.get_info();
        parts.push(format!("{} ({:?})", info.name, info.backend));
    }
    if let Some(card) = graphics_cards().into_iter().next() {
        parts.extend(card.driver);
        parts.extend(card.id);
    }
    (!parts.is_empty()).then(|| parts.join(", "))
}

fn plasma_version() -> Option<String> {
    let output = Command::new("plasmashell").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.split_whitespace().last()?.to_owned())
}

fn read_os_name() -> Option<String> {
    os_name(&std::fs::read_to_string("/etc/os-release").ok()?)
}

fn read_trimmed(path: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

fn os_name(os_release: &str) -> Option<String> {
    let line = os_release
        .lines()
        .find(|line| line.starts_with("PRETTY_NAME="))?;
    Some(
        line.trim_start_matches("PRETTY_NAME=")
            .trim_matches('"')
            .to_owned(),
    )
}

fn cpu_name() -> Option<String> {
    let info = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    let line = info.lines().find(|line| line.starts_with("model name"))?;
    Some(line.split_once(':')?.1.trim().to_owned())
}

fn memory_gib() -> Option<f64> {
    let info = std::fs::read_to_string("/proc/meminfo").ok()?;
    let line = info.lines().find(|line| line.starts_with("MemTotal:"))?;
    let kib: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kib / 1024.0 / 1024.0)
}

struct GraphicsCard {
    driver: Option<String>,
    id: Option<String>,
}

fn graphics_cards() -> Vec<GraphicsCard> {
    let mut cards = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/class/drm") else {
        return cards;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let Ok(uevent) = std::fs::read_to_string(entry.path().join("device/uevent")) else {
            continue;
        };
        let field = |key: &str| {
            uevent
                .lines()
                .find_map(|line| line.strip_prefix(key).map(str::to_owned))
        };
        cards.push(GraphicsCard {
            driver: field("DRIVER="),
            id: field("PCI_ID="),
        });
    }
    cards
}

fn displays(frame: &eframe::Frame) -> Vec<String> {
    let Some(window) = frame.winit_window() else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    for monitor in window.available_monitors() {
        let name = monitor.name().unwrap_or_else(|| "unknown".to_owned());
        let size = monitor.size();
        let refresh = monitor
            .refresh_rate_millihertz()
            .map(|millihertz| format!(" @ {} Hz", f64::from(millihertz) / 1000.0))
            .unwrap_or_default();
        let line = format!("  {name}: {}x{}{refresh}", size.width, size.height);
        // winit can list an output more than once.
        if !lines.contains(&line) {
            lines.push(line);
        }
    }
    if !lines.is_empty() {
        // The monitor API reports whole-number scales; the window has the
        // fractional one the UI actually uses.
        lines.push(format!("  Scale: {:.2}", window.scale_factor()));
    }
    lines
}

fn log_level_combo(ui: &mut egui::Ui, level: &mut LogLevel) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt("log-level")
        .selected_text(level.as_str())
        .show_ui(ui, |ui| {
            for candidate in LogLevel::ALL {
                if ui
                    .selectable_value(level, candidate, candidate.as_str())
                    .changed()
                {
                    changed = true;
                }
            }
        })
        .response
        .on_hover_text("Log level: applies immediately and saves to the config file.");
    changed
}

fn paths() -> [(&'static str, Option<PathBuf>); 4] {
    [
        ("Config", Config::path()),
        ("History", State::path()),
        (LOG_ROW_LABEL, logging::directory()),
        ("Binary", std::env::current_exe().ok()),
    ]
}

fn path_value(ui: &mut egui::Ui, path: Option<PathBuf>) {
    let Some(path) = path else {
        ui.weak("unavailable");
        return;
    };

    let text = path.display().to_string();
    let font = egui::FontId::monospace(PATH_FONT_SIZE);
    let color = ui.visuals().hyperlink_color;
    let mut job = egui::text::LayoutJob::simple(text, font, color, f32::INFINITY);
    job.wrap =
        egui::text::TextWrapping::truncate_at_width(ui.available_width() - 2.0 * PATH_PADDING);
    let galley = ui.painter().layout_job(job);

    // The row is hard to hit at glyph height, so pad the hit area vertically.
    let size = galley.size() + egui::vec2(2.0, 2.0) * PATH_PADDING;
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    // A long path is truncated, so the hover carries the whole thing.
    let response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(format!("{}\nClick to open its folder.", path.display()));
    let underline = if response.hovered() {
        egui::Stroke::new(ui.style().visuals.widgets.hovered.fg_stroke.width, color)
    } else {
        egui::Stroke::NONE
    };
    ui.painter().add(
        egui::epaint::TextShape::new(
            rect.left_top() + egui::vec2(PATH_PADDING, PATH_PADDING),
            galley,
            color,
        )
        .with_underline(underline),
    );
    if response.clicked() {
        open_path(&path);
    }
}

/// Open the folder a path lives in: launching the file itself is not useful for
/// a log directory, a config file or the binary.
fn open_path(path: &Path) {
    let folder = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    open_target(folder.as_os_str());
}

fn open_url(url: &str) {
    open_target(std::ffi::OsStr::new(url));
}

/// One spawn-and-reap for folders and URLs: the opener hands off and returns.
fn open_target(target: &std::ffi::OsStr) {
    match Command::new(OPENER).arg(target).spawn() {
        Ok(mut child) => {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(error) => {
            tracing::warn!(%error, target = %Path::new(target).display(), "could not open it");
        }
    }
}

fn logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let image = image::load_from_memory(ICON_PNG).ok()?.into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &image.into_raw());
    Some(ctx.load_texture(
        "colorpickle-logo",
        color_image,
        egui::TextureOptions::LINEAR,
    ))
}

#[cfg(test)]
mod tests {
    use super::os_name;

    #[test]
    fn reads_the_pretty_os_name() {
        assert_eq!(
            os_name("NAME=\"CachyOS\"\nPRETTY_NAME=\"CachyOS Linux\"\n"),
            Some("CachyOS Linux".to_owned())
        );
        assert_eq!(os_name("NAME=\"CachyOS\"\n"), None);
    }
}
