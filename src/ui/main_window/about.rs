use std::path::{Path, PathBuf};
use std::process::Command;

use eframe::egui;

use crate::capture;
use crate::config::{Config, LogLevel};
use crate::logging;
use crate::state::State;
use crate::ui::{ICON_PNG, WINDOW_TITLE, widgets};

const WINDOW_ID: &str = "colorpickle-about";
const OPENER: &str = "xdg-open";
const LOGO_SIZE: f32 = 144.0;
const LOGO_SPACING: f32 = 12.0;
const SECTION_SPACING: f32 = 14.0;
const ROW_SPACING: f32 = 14.0;
const NAME_FONT_SIZE: f32 = 22.0;
const PATH_FONT_SIZE: f32 = 13.0;
const SECTION_TITLE_FONT_SIZE: f32 = 15.0;
const WINDOW_WIDTH: f32 = 480.0;
const LOG_ROW_LABEL: &str = "Logs";
const PATH_PADDING: f32 = 2.0;
/// Paths sit under their title like a list, not flush with it.
const LINK_INDENT: f32 = 12.0;
/// egui bundles no bold font, so titles are overpainted by this much to gain weight.
const BOLD_SMUDGE: f32 = 0.5;

#[derive(Default)]
pub struct About {
    open: bool,
    logo: Option<egui::TextureHandle>,
}

impl About {
    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Returns `true` when the log level changed.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        frame: &eframe::Frame,
        config: &mut Config,
    ) -> bool {
        if !self.open {
            return false;
        }
        if self.logo.is_none() {
            self.logo = logo_texture(ctx);
        }

        let mut open = self.open;
        let mut changed = false;
        egui::Window::new(format!("About {WINDOW_TITLE}"))
            .id(egui::Id::unique(WINDOW_ID))
            .collapsible(false)
            .resizable(false)
            .default_width(WINDOW_WIDTH)
            .open(&mut open)
            .show(ctx, |ui| {
                changed = self.contents(ui, &mut config.log_level, frame);
            });
        self.open = open;
        changed
    }

    fn contents(&self, ui: &mut egui::Ui, level: &mut LogLevel, frame: &eframe::Frame) -> bool {
        ui.vertical_centered(|ui| {
            if let Some(logo) = &self.logo {
                ui.add(egui::Image::new(logo).fit_to_exact_size(egui::vec2(LOGO_SIZE, LOGO_SIZE)));
                ui.add_space(LOGO_SPACING);
            }
            ui.label(
                egui::RichText::new(WINDOW_TITLE)
                    .size(NAME_FONT_SIZE)
                    .strong(),
            );
            version_row(ui, frame);
        });

        ui.add_space(SECTION_SPACING);
        ui.separator();
        ui.add_space(SECTION_SPACING);

        let mut changed = false;
        for (label, path) in paths() {
            ui.horizontal(|ui| {
                section_title(ui, label);
                if label == LOG_ROW_LABEL {
                    changed = log_level_combo(ui, level);
                }
            });
            ui.horizontal(|ui| {
                ui.add_space(LINK_INDENT);
                path_value(ui, path);
            });
            ui.add_space(ROW_SPACING);
        }
        changed
    }
}

fn section_title(ui: &mut egui::Ui, text: &str) {
    let font = egui::FontId::proportional(SECTION_TITLE_FONT_SIZE);
    let color = ui.visuals().strong_text_color();
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, color);
    let (rect, _) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
    let painter = ui.painter();
    painter.galley(rect.left_top(), galley.clone(), color);
    painter.galley(
        rect.left_top() + egui::vec2(BOLD_SMUDGE, 0.0),
        galley,
        color,
    );
}

fn version_row(ui: &mut egui::Ui, frame: &eframe::Frame) {
    let text = format!("Version {}", env!("CARGO_PKG_VERSION"));
    ui.horizontal(|ui| {
        let font = egui::TextStyle::Body.resolve(ui.style());
        let width = ui
            .painter()
            .layout_no_wrap(text.clone(), font, ui.visuals().text_color())
            .size()
            .x
            + ui.spacing().item_spacing.x
            + widgets::COPY_BUTTON_SIZE;
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        ui.label(text);
        if widgets::copy_button(ui, "Copy environment info") {
            ui.ctx().copy_text(environment_info(frame));
        }
    });
}

fn environment_info(frame: &eframe::Frame) -> String {
    let mut lines = vec![format!("{WINDOW_TITLE} {}", env!("CARGO_PKG_VERSION"))];
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
    if let Some(gpus) = gpus() {
        lines.push(format!("GPU: {gpus}"));
    }
    let displays = displays(frame);
    if !displays.is_empty() {
        lines.push("Displays:".to_owned());
        lines.extend(displays);
    }
    let backends: Vec<&str> = capture::available_sources()
        .iter()
        .map(|source| source.label())
        .collect();
    if !backends.is_empty() {
        lines.push(format!("Capture backends: {}", backends.join(", ")));
    }
    lines.join("\n")
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

/// Graphics drivers and PCI ids straight from sysfs: a driver name is what
/// matters for a capture or rendering bug, not the marketing name.
fn gpus() -> Option<String> {
    let mut names = Vec::new();
    for card in graphics_cards() {
        let name = card
            .slot
            .as_deref()
            .and_then(pci_name)
            .or_else(|| card.id.clone())
            .unwrap_or_else(|| "unknown".to_owned());
        let name = match (card.driver, card.id) {
            (Some(driver), Some(id)) => format!("{name} ({driver}, {id})"),
            (Some(driver), None) => format!("{name} ({driver})"),
            (None, Some(id)) => format!("{name} ({id})"),
            (None, None) => name,
        };
        names.push(name);
    }
    (!names.is_empty()).then(|| names.join(", "))
}

struct GraphicsCard {
    slot: Option<String>,
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
            slot: field("PCI_SLOT_NAME="),
            driver: field("DRIVER="),
            id: field("PCI_ID="),
        });
    }
    cards
}

/// The card's marketing name comes from `pci.ids`, which lspci already ships;
/// sysfs only holds driver and PCI ids.
fn pci_name(slot: &str) -> Option<String> {
    let output = Command::new("lspci")
        .args(["-mm", "-s", slot])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_pci_name(&String::from_utf8(output.stdout).ok()?)
}

/// `lspci -mm` quotes the class, vendor, device and, when the board has one,
/// the subsystem pair. The subsystem name is the precise board name.
fn parse_pci_name(line: &str) -> Option<String> {
    let quoted: Vec<&str> = line.split('"').skip(1).step_by(2).collect();
    let board = quoted.get(4).copied().unwrap_or_default();
    let device = quoted.get(2).copied().unwrap_or_default();
    let name = if board.is_empty() { device } else { board };
    (!name.is_empty()).then(|| name.to_owned())
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
    let response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Click to open its folder.");
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
    match Command::new(OPENER).arg(folder).spawn() {
        Ok(mut child) => {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(error) => {
            tracing::warn!(%error, path = %folder.display(), "could not open the folder");
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
    use super::{os_name, parse_pci_name};

    #[test]
    fn reads_the_pretty_os_name() {
        assert_eq!(
            os_name("NAME=\"CachyOS\"\nPRETTY_NAME=\"CachyOS Linux\"\n"),
            Some("CachyOS Linux".to_owned())
        );
        assert_eq!(os_name("NAME=\"CachyOS\"\n"), None);
    }

    #[test]
    fn prefers_the_board_name_from_lspci() {
        assert_eq!(
            parse_pci_name(
                "0a:00.0 \"VGA compatible controller\" \"Advanced Micro Devices, Inc. [AMD/ATI]\" \"Navi 31 [Radeon RX 7900 XT/7900 XTX/7900 GRE/7900M]\" -rcc -p00 \"Sapphire Technology Limited\" \"PULSE RX 7900 XTX\"\n"
            ),
            Some("PULSE RX 7900 XTX".to_owned())
        );
        assert_eq!(
            parse_pci_name(
                "00:02.0 \"VGA compatible controller\" \"Intel Corporation\" \"Raptor Lake-P [Iris Xe Graphics]\" -r00 -p00\n"
            ),
            Some("Raptor Lake-P [Iris Xe Graphics]".to_owned())
        );
        assert_eq!(parse_pci_name(""), None);
    }
}
