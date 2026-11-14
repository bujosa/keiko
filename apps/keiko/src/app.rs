//! The Keiko window: top bar, toolbar, the dojos in a grid or a list, and the footer.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, Margin, RichText, Sense, Stroke, Ui, pos2, vec2};
use keiko_core::{App, Catalog, Latest, State, format};
use keiko_native::install::Phase;
use keiko_native::store::{Feed, Settings, Sort, Theme, View};
use keiko_native::{Installed, Paths, launch};

use crate::jobs::{Jobs, Msg};
use crate::theme::{self, Tokens};
use crate::widgets::{self, Icon, Kind};

/// Re-check GitHub when the last check is older than this.
const STALE: u64 = 3600;
const MORE_W: f32 = 40.0;

pub struct Entry {
    pub app: App,
    pub installed: Option<Installed>,
    pub latest: Option<Latest>,
    pub checking: bool,
    pub error: Option<String>,
    pub busy: Option<Busy>,
    /// Keiko itself (shown as a banner, not a card).
    pub launcher: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Busy {
    Phase(Phase),
    Uninstalling,
}

impl Entry {
    pub fn state(&self) -> State {
        let installed = self.installed.as_ref().map(|i| i.semver());
        if let Some(None) = installed {
            // Installed with a version Keiko cannot compare: never offer a downgrade.
            return State::Installed;
        }
        let latest = self.latest.as_ref().and_then(Latest::semver);
        let has_asset = self.latest.as_ref().is_some_and(|l| l.asset.is_some());
        State::of(installed.flatten().as_ref(), latest.as_ref(), has_asset)
    }

    fn busy_label(&self) -> Option<String> {
        Some(match self.busy? {
            Busy::Phase(Phase::Checksums) => "Reading checksums".into(),
            Busy::Phase(Phase::Downloading { done, total }) if total > 0 => {
                format!("Downloading {}%", (done * 100 / total).min(100))
            }
            Busy::Phase(Phase::Downloading { .. }) => "Downloading".into(),
            Busy::Phase(Phase::Verifying) => "Verifying SHA-256".into(),
            Busy::Phase(Phase::Installing) => "Installing".into(),
            Busy::Uninstalling => "Moving to Trash".into(),
        })
    }

    fn matches(&self, q: &str) -> bool {
        let q = q.trim().to_lowercase();
        q.is_empty()
            || [&self.app.name, &self.app.tagline, &self.app.description].iter().any(|s| s.to_lowercase().contains(&q))
    }
}

enum Action {
    Install(String),
    Open(String),
    Reveal(String),
    Url(String, String),
    Check(String),
    AskUninstall(String),
}

pub struct KeikoApp {
    paths: Paths,
    settings: Settings,
    feed: Feed,
    entries: Vec<Entry>,
    jobs: Jobs,
    t: Tokens,
    applied_dark: Option<bool>,
    search: String,
    textures: HashMap<String, egui::TextureHandle>,
    toasts: Vec<(String, bool, Instant)>,
    confirm_uninstall: Option<String>,
    pending: usize,
    was_focused: bool,
    /// Tallest card measured last frame: every card in the grid gets this height.
    card_h: f32,
    next_card_h: f32,
    /// Keiko replaced its own bundle: open it again and quit.
    restart: Option<PathBuf>,
    startup_error: Option<String>,
    /// `KEIKO_SCREENSHOT=<png>`: capture the window once the first check settles, then quit.
    shot: Option<(PathBuf, Instant, bool)>,
}

fn app_png(id: &str) -> Option<&'static [u8]> {
    match id {
        "poddojo" => Some(include_bytes!("../../../assets/apps/poddojo.png")),
        _ => None,
    }
}

/// The `.app` bundle Keiko runs from (`KEIKO_SELF_BUNDLE` overrides it for development). `None`
/// when it runs outside a bundle (`cargo run`): then it does not offer to update itself.
fn running_bundle() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("KEIKO_SELF_BUNDLE") {
        return Some(PathBuf::from(p));
    }
    let exe = std::env::current_exe().ok()?;
    exe.ancestors().find(|a| a.extension().is_some_and(|x| x == "app")).map(std::path::Path::to_path_buf)
}

static LOGO: &[u8] = include_bytes!("../../../assets/app-icon/keiko-256.png");

fn texture(ctx: &egui::Context, name: &str, png: &[u8]) -> Option<egui::TextureHandle> {
    let img = image::load_from_memory_with_format(png, image::ImageFormat::Png).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture(name, color, egui::TextureOptions::LINEAR))
}

/// RGBA pixels for the window icon.
pub fn icon_rgba() -> Option<(Vec<u8>, u32, u32)> {
    let img = image::load_from_memory_with_format(LOGO, image::ImageFormat::Png).ok()?.to_rgba8();
    let (w, h) = (img.width(), img.height());
    Some((img.into_raw(), w, h))
}

impl KeikoApp {
    pub fn new(ctx: &egui::Context) -> KeikoApp {
        theme::fonts(ctx);
        let (paths, startup_error) = match Paths::detect() {
            Ok(p) => (p, None),
            Err(e) => (Paths::for_tests(&std::env::temp_dir().join("keiko")), Some(e)),
        };
        let (catalog, catalog_error) = match Catalog::bundled() {
            Ok(c) => (c, None),
            Err(e) => (Catalog { apps: vec![], launcher: None }, Some(e)),
        };
        let settings = Settings::load(&paths);
        let feed = Feed::load(&paths);
        let launcher = catalog.launcher.clone();
        let mut entries: Vec<Entry> = catalog
            .apps
            .into_iter()
            .map(|app| Entry {
                latest: feed.latest.get(&app.id).cloned(),
                app,
                installed: None,
                checking: false,
                error: None,
                busy: None,
                launcher: false,
            })
            .collect();
        if let (Some(app), Some(path)) = (launcher, running_bundle()) {
            let version = std::env::var("KEIKO_SELF_VERSION").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").into());
            entries.push(Entry {
                latest: feed.latest.get(&app.id).cloned(),
                app,
                installed: Some(Installed { path, version }),
                checking: false,
                error: None,
                busy: None,
                launcher: true,
            });
        }
        let mut textures = HashMap::new();
        if let Some(tex) = texture(ctx, "keiko", LOGO) {
            textures.insert("keiko".to_string(), tex);
        }
        let shot = std::env::var_os("KEIKO_SCREENSHOT").map(|p| (PathBuf::from(p), Instant::now(), false));
        let mut app = KeikoApp {
            paths,
            settings,
            feed,
            entries,
            jobs: Jobs::new(ctx),
            t: Tokens::DARK,
            applied_dark: None,
            search: String::new(),
            textures,
            toasts: vec![],
            confirm_uninstall: None,
            pending: 0,
            was_focused: true,
            card_h: 0.0,
            next_card_h: 0.0,
            restart: None,
            startup_error: startup_error.or(catalog_error),
            shot,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        for e in &mut self.entries {
            if e.busy.is_none() {
                e.checking = true;
                self.pending += 1;
                if e.launcher {
                    self.jobs.check_latest(&e.app);
                } else {
                    self.jobs.check(&self.paths, &e.app);
                }
            }
        }
    }

    fn entry(&mut self, id: &str) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|e| e.app.id == id)
    }

    fn toast(&mut self, text: impl Into<String>, ok: bool) {
        self.toasts.push((text.into(), ok, Instant::now()));
    }

    /// A macOS notification, once per app and version.
    fn announce(&mut self, id: &str, name: &str, version: &str, launcher: bool) {
        if self.feed.notified.get(id).is_some_and(|v| v == version) {
            return;
        }
        self.feed.notified.insert(id.to_string(), version.to_string());
        let subtitle = format!("{name} {version} is available");
        let message = if launcher { "Update and restart from Keiko." } else { "Open Keiko to update it." };
        self.jobs.act(id, move || keiko_native::notify::notify("Keiko", &subtitle, message));
    }

    fn handle(&mut self, msg: Msg) {
        match msg {
            Msg::Detected(id, found) => {
                if let Some(e) = self.entry(&id) {
                    e.installed = found;
                }
            }
            Msg::Latest(id, result) => {
                self.pending = self.pending.saturating_sub(1);
                let mut announce = None;
                if let Some(e) = self.entry(&id) {
                    e.checking = false;
                    match result {
                        Ok(latest) => {
                            e.error = None;
                            e.latest = latest.clone();
                            if e.state() == State::UpdateAvailable
                                && let Some(l) = &latest
                            {
                                announce = Some((e.app.name.clone(), l.version.clone(), e.launcher));
                            }
                            match latest {
                                Some(l) => self.feed.latest.insert(id.clone(), l),
                                None => self.feed.latest.remove(&id),
                            };
                        }
                        Err(err) => e.error = Some(err),
                    }
                }
                if let Some((name, version, launcher)) = announce {
                    self.announce(&id, &name, &version, launcher);
                }
                if self.pending == 0 {
                    self.feed.checked_at = keiko_core::now();
                    if let Err(e) = self.feed.save(&self.paths) {
                        self.toast(format!("Could not save the release cache: {e}"), false);
                    }
                }
            }
            Msg::Phase(id, phase) => {
                if let Some(e) = self.entry(&id) {
                    e.busy = Some(Busy::Phase(phase));
                }
            }
            Msg::Installed(id, result) => {
                let mut note = None;
                let mut restart = None;
                if let Some(e) = self.entry(&id) {
                    e.busy = None;
                    match result {
                        Ok(i) if e.launcher => {
                            restart = Some(i.path.clone());
                            e.installed = Some(i);
                        }
                        Ok(i) => {
                            e.error = None;
                            note = Some((format!("{} {} is installed", e.app.name, i.version), true));
                            e.installed = Some(i);
                        }
                        Err(err) => {
                            note = Some((format!("{}: {err}", e.app.name), false));
                            e.error = Some(err);
                        }
                    }
                }
                if let Some((text, ok)) = note {
                    self.toast(text, ok);
                }
                if restart.is_some() {
                    self.restart = restart;
                }
            }
            Msg::Uninstalled(id, result) => {
                let mut note = None;
                if let Some(e) = self.entry(&id) {
                    e.busy = None;
                    match result {
                        Ok(_) => {
                            note = Some((format!("{} was moved to the Trash", e.app.name), true));
                            e.installed = None;
                        }
                        Err(err) => note = Some((err, false)),
                    }
                }
                if let Some((text, ok)) = note {
                    self.toast(text, ok);
                }
            }
            Msg::Failed(id, err) => {
                let name = self.entries.iter().find(|e| e.app.id == id).map(|e| e.app.name.clone());
                self.toast(name.map(|n| format!("{n}: {err}")).unwrap_or(err), false);
            }
        }
    }

    fn act(&mut self, action: Action) {
        let find = |s: &Self, id: &str| s.entries.iter().position(|e| e.app.id == id);
        match action {
            Action::Install(id) => {
                let Some(i) = find(self, &id) else { return };
                let e = &self.entries[i];
                if e.busy.is_some() {
                    return;
                }
                let Some(latest) = e.latest.clone() else { return };
                self.jobs.install(&self.paths, &e.app, &latest, e.installed.as_ref());
                let e = &mut self.entries[i];
                e.busy = Some(Busy::Phase(Phase::Checksums));
                e.error = None;
            }
            Action::Open(id) => {
                if let Some(path) =
                    find(self, &id).and_then(|i| self.entries[i].installed.as_ref().map(|x| x.path.clone()))
                {
                    self.jobs.act(&id, move || launch::open_app(&path));
                }
            }
            Action::Reveal(id) => {
                if let Some(path) =
                    find(self, &id).and_then(|i| self.entries[i].installed.as_ref().map(|x| x.path.clone()))
                {
                    self.jobs.act(&id, move || launch::reveal(&path));
                }
            }
            Action::Url(id, url) => self.jobs.act(&id, move || launch::open_url(&url)),
            Action::Check(id) => {
                if let Some(i) = find(self, &id) {
                    self.entries[i].checking = true;
                    self.pending += 1;
                    self.jobs.check(&self.paths, &self.entries[i].app);
                }
            }
            Action::AskUninstall(id) => self.confirm_uninstall = Some(id),
        }
    }

    // ---------------------------------------------------------------------------------------

    pub fn frame(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        for msg in self.jobs.poll() {
            self.handle(msg);
        }
        self.apply_theme(&ctx);
        self.watch_focus(&ctx);
        if self.pending == 0
            && self.entries.iter().all(|e| e.busy.is_none())
            && keiko_core::now().saturating_sub(self.feed.checked_at) > STALE
        {
            self.refresh();
        }

        self.card_h = self.next_card_h;
        self.next_card_h = 0.0;
        let mut actions = vec![];
        self.top_bar(ui);
        self.footer(ui);
        let t = self.t;
        egui::CentralPanel::default().frame(egui::Frame::new().fill(t.bg).inner_margin(Margin::symmetric(26, 0))).show(
            ui,
            |ui| {
                ui.add_space(22.0);
                self.toolbar(ui, &mut actions);
                ui.add_space(22.0);
                self.banner(ui, &mut actions);
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    if let Some(e) = &self.startup_error {
                        ui.colored_label(t.bad, e);
                    }
                    self.sections(ui, &mut actions);
                    ui.add_space(26.0);
                });
            },
        );
        for a in actions {
            self.act(a);
        }
        if let Some(path) = self.restart.take() {
            match launch::relaunch(&path) {
                Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                Err(e) => self.toast(format!("Keiko was updated; open it again ({e})"), false),
            }
        }
        self.confirm(&ctx);
        self.toasts(&ctx);
        self.screenshot(&ctx);
        if self.pending > 0 || self.entries.iter().any(|e| e.busy.is_some()) {
            ctx.request_repaint_after(Duration::from_millis(200));
        } else {
            // Keep "checked N min ago" honest.
            ctx.request_repaint_after(Duration::from_secs(30));
        }
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        let dark = match self.settings.theme {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::Auto => ctx.system_theme() != Some(egui::Theme::Light),
        };
        if self.applied_dark != Some(dark) {
            self.t = if dark { Tokens::DARK } else { Tokens::LIGHT };
            theme::apply(ctx, &self.t);
            self.applied_dark = Some(dark);
        }
    }

    /// Coming back to the window re-reads what is installed and each dojo's progress.
    fn watch_focus(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(|i| i.focused);
        if focused && !self.was_focused {
            for e in self.entries.iter().filter(|e| e.busy.is_none() && !e.launcher) {
                self.jobs.local(&self.paths, &e.app);
            }
        }
        self.was_focused = focused;
    }

    fn save_settings(&mut self) {
        if let Err(e) = self.settings.save(&self.paths) {
            self.toast(format!("Could not save settings: {e}"), false);
        }
    }

    // --- top bar, toolbar, footer -----------------------------------------------------------

    fn top_bar(&mut self, ui: &mut Ui) {
        let t = self.t;
        egui::Panel::top("top")
            .exact_size(64.0)
            .frame(
                egui::Frame::new()
                    .fill(t.panel)
                    .inner_margin(Margin { left: 84, right: 14, top: 0, bottom: 0 })
                    .stroke(Stroke::new(1.0, t.border)),
            )
            .show(ui, |ui| {
                // The title bar is hidden: the top bar moves the window, a double click zooms it.
                let drag = ui.interact(ui.max_rect(), egui::Id::new("titlebar"), Sense::click_and_drag());
                if drag.drag_started_by(egui::PointerButton::Primary) {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if drag.double_clicked() {
                    let zoomed = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!zoomed));
                }
                ui.horizontal_centered(|ui| {
                    widgets::mark(ui, &t, 30.0);
                    ui.add_space(4.0);
                    let mut job = LayoutJob::default();
                    job.append(
                        "KEIKO",
                        0.0,
                        TextFormat {
                            font_id: FontId::new(23.0, theme::display()),
                            color: t.text,
                            extra_letter_spacing: 4.0,
                            ..Default::default()
                        },
                    );
                    ui.label(job);
                    ui.add_space(14.0);
                    let (r, _) = ui.allocate_exact_size(vec2(1.0, 64.0), Sense::hover());
                    ui.painter().rect_filled(r, 0.0, t.border);
                    ui.add_space(14.0);
                    widgets::label(ui, "Dojo app suite", 12.5, t.muted);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let next = match self.settings.theme {
                            Theme::Auto => Theme::Dark,
                            Theme::Dark => Theme::Light,
                            Theme::Light => Theme::Auto,
                        };
                        let now = match self.settings.theme {
                            Theme::Auto => "Theme: follows macOS",
                            Theme::Dark => "Theme: dark",
                            Theme::Light => "Theme: light",
                        };
                        if widgets::icon_button(ui, &t, Icon::Theme, false, &format!("{now} (click to change)"))
                            .clicked()
                        {
                            self.settings.theme = next;
                            self.applied_dark = None;
                            self.save_settings();
                        }
                    });
                });
            });
    }

    fn toolbar(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let t = self.t;
        ui.horizontal(|ui| {
            let spin = self.pending > 0;
            let r = widgets::icon_button(ui, &t, Icon::Refresh, false, "Check GitHub for new releases");
            if spin {
                ui.painter().rect_filled(r.rect.shrink(1.0), 1.0, t.bg.gamma_multiply(0.6));
                ui.put(r.rect, egui::Spinner::new().size(16.0).color(t.text));
            } else if r.clicked() {
                self.refresh();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::icon_button(ui, &t, Icon::List, self.settings.view == View::List, "List").clicked() {
                    self.settings.view = View::List;
                    self.save_settings();
                }
                if widgets::icon_button(ui, &t, Icon::Grid, self.settings.view == View::Grid, "Grid").clicked() {
                    self.settings.view = View::Grid;
                    self.save_settings();
                }
                ui.add_space(6.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text("Search dojos")
                        .desired_width(240.0)
                        .margin(vec2(12.0, 11.0))
                        .font(FontId::new(15.0, egui::FontFamily::Proportional)),
                );
                ui.add_space(6.0);
                let before = self.settings.sort;
                let text = |s: Sort| match s {
                    Sort::Name => "Name (A–Z)",
                    Sort::Updated => "Recently released",
                    Sort::Status => "Status",
                };
                egui::ComboBox::from_id_salt("sort")
                    .selected_text(RichText::new(text(self.settings.sort)).size(15.0))
                    .width(200.0)
                    .height(240.0)
                    .show_ui(ui, |ui| {
                        for s in [Sort::Name, Sort::Updated, Sort::Status] {
                            ui.selectable_value(&mut self.settings.sort, s, text(s));
                        }
                    });
                if self.settings.sort != before {
                    self.save_settings();
                }
                widgets::label(ui, "Sort", 12.0, t.muted);
                let updates: Vec<String> = self
                    .entries
                    .iter()
                    .filter(|e| e.state() == State::UpdateAvailable && e.busy.is_none() && !e.launcher)
                    .map(|e| e.app.id.clone())
                    .collect();
                if !updates.is_empty() {
                    ui.add_space(10.0);
                    let label = match updates.len() {
                        1 => "Update 1 app".to_string(),
                        n => format!("Update {n} apps"),
                    };
                    if widgets::button(ui, &t, &label, Kind::Primary, 0.0, true).clicked() {
                        actions.extend(updates.into_iter().map(Action::Install));
                    }
                }
            });
        });
    }

    fn footer(&mut self, ui: &mut Ui) {
        let t = self.t;
        egui::Panel::bottom("footer")
            .exact_size(38.0)
            .frame(
                egui::Frame::new()
                    .fill(t.panel)
                    .inner_margin(Margin::symmetric(26, 0))
                    .stroke(Stroke::new(1.0, t.border)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    let status = if self.pending > 0 {
                        "Checking GitHub…".to_string()
                    } else if self.feed.checked_at == 0 {
                        "Not checked yet".to_string()
                    } else {
                        format!("Checked {}", format::ago(keiko_core::now().saturating_sub(self.feed.checked_at)))
                    };
                    widgets::label(
                        ui,
                        &format!("{status} · GitHub releases · macOS {}", keiko_core::arch()),
                        11.0,
                        t.faint,
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let r = ui
                            .add(
                                egui::Label::new(widgets::spaced(
                                    concat!("Keiko v", env!("CARGO_PKG_VERSION"), " · open source"),
                                    11.0,
                                    t.faint,
                                    false,
                                ))
                                .sense(Sense::click()),
                            )
                            .on_hover_text("github.com/bujosa/keiko");
                        if r.clicked() {
                            self.jobs.act("keiko", || launch::open_url("https://github.com/bujosa/keiko"));
                        }
                    });
                });
            });
    }

    // --- the dojos ----------------------------------------------------------------------------

    fn ordered(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.entries.len())
            .filter(|i| !self.entries[*i].launcher && self.entries[*i].matches(&self.search))
            .collect();
        let e = &self.entries;
        match self.settings.sort {
            Sort::Name => idx.sort_by_key(|i| e[*i].app.name.to_lowercase()),
            Sort::Updated => idx.sort_by(|a, b| {
                let d = |i: usize| e[i].latest.as_ref().and_then(|l| l.published_at.clone()).unwrap_or_default();
                d(*b).cmp(&d(*a))
            }),
            Sort::Status => idx.sort_by_key(|i| (e[*i].state(), e[*i].app.name.to_lowercase())),
        }
        idx
    }

    fn sections(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let t = self.t;
        let order = self.ordered();
        if order.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(format!("No dojo matches “{}”", self.search.trim())).size(16.0).color(t.muted));
            });
            return;
        }
        let (installed, other): (Vec<usize>, Vec<usize>) =
            order.into_iter().partition(|i| self.entries[*i].installed.is_some());
        let mut first = true;
        for (title, group) in [("Installed", installed), ("Available", other)] {
            if group.is_empty() {
                continue;
            }
            if !first {
                ui.add_space(34.0);
            }
            first = false;
            widgets::section_title(ui, &t, &format!("{title} · {}", group.len()));
            ui.add_space(14.0);
            match self.settings.view {
                View::Grid => self.grid(ui, &group, actions),
                View::List => self.list(ui, &group, actions),
            }
        }
    }

    fn grid(&mut self, ui: &mut Ui, group: &[usize], actions: &mut Vec<Action>) {
        let gap = 16.0;
        let width = ui.available_width();
        let cols = ((width + gap) / (380.0 + gap)).floor().clamp(1.0, 4.0) as usize;
        let card_w = ((width - gap * (cols as f32 - 1.0)) / cols as f32).floor();
        for row in group.chunks(cols) {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for i in row {
                    ui.allocate_ui_with_layout(vec2(card_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                        self.card(ui, *i, card_w, actions);
                    });
                }
            });
            ui.add_space(gap);
        }
    }

    /// Keiko's own update, above the catalog.
    fn banner(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let Some(i) = self.entries.iter().position(|e| e.launcher) else { return };
        let e = &self.entries[i];
        let busy = e.busy_label();
        if e.state() != State::UpdateAvailable && busy.is_none() {
            return;
        }
        let t = self.t;
        let current = e.installed.as_ref().map(|i| i.version.clone()).unwrap_or_default();
        let latest = e.latest.clone();
        let version = latest.as_ref().map(|l| l.version.clone()).unwrap_or_default();
        let id = e.app.id.clone();
        let error = e.error.clone();
        let width = ui.available_width();
        egui::Frame::new()
            .fill(t.seal.gamma_multiply(0.10))
            .stroke(Stroke::new(1.0, t.seal.gamma_multiply(0.55)))
            .inner_margin(Margin::symmetric(18, 12))
            .show(ui, |ui| {
                ui.set_width(width - 38.0);
                ui.allocate_ui_with_layout(
                    vec2(width - 38.0, 42.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        widgets::mark(ui, &t, 24.0);
                        ui.label(widgets::spaced(&format!("Keiko {version} is available"), 12.5, t.text, true));
                        let note = match &error {
                            Some(err) => RichText::new(err).size(14.0).color(t.bad),
                            None => RichText::new(format!("You have {current}.")).size(14.0).color(t.muted),
                        };
                        ui.label(note);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            match &busy {
                                Some(label) => {
                                    widgets::button(ui, &t, label, Kind::Primary, 220.0, false);
                                }
                                None => {
                                    if widgets::button(ui, &t, "Update and restart", Kind::Primary, 0.0, true).clicked()
                                    {
                                        actions.push(Action::Install(id.clone()));
                                    }
                                    if let Some(l) = &latest
                                        && !l.html_url.is_empty()
                                        && widgets::button(ui, &t, "What's new", Kind::Secondary, 0.0, true).clicked()
                                    {
                                        actions.push(Action::Url(id.clone(), l.html_url.clone()));
                                    }
                                }
                            }
                        });
                    },
                );
            });
        ui.add_space(24.0);
    }

    fn icon(&mut self, ui: &mut Ui, i: usize, size: f32) {
        let id = self.entries[i].app.id.clone();
        if !self.textures.contains_key(&id)
            && let Some(png) = app_png(&id)
            && let Some(tex) = texture(ui.ctx(), &id, png)
        {
            self.textures.insert(id.clone(), tex);
        }
        match self.textures.get(&id) {
            Some(tex) => {
                ui.add(egui::Image::new((tex.id(), vec2(size, size))));
            }
            None => {
                let [r, g, b] = self.entries[i].app.color;
                widgets::initial_tile(ui, Color32::from_rgb(r, g, b), &self.entries[i].app.name, size);
            }
        }
    }

    fn name_job(&self, e: &Entry, size: f32) -> LayoutJob {
        let [r, g, b] = e.app.color;
        let (head, tail) = e.app.name_parts();
        let mut job = LayoutJob::default();
        let fmt = |color| TextFormat { font_id: FontId::new(size, theme::display()), color, ..Default::default() };
        job.append(head, 0.0, fmt(self.t.text));
        job.append(tail, 0.0, fmt(Color32::from_rgb(r, g, b)));
        job
    }

    fn version_job(&self, e: &Entry) -> LayoutJob {
        let t = self.t;
        let latest = e.latest.as_ref().map(|l| l.version.clone());
        match (&e.installed, e.state()) {
            (Some(i), State::UpdateAvailable) => {
                let mut job = widgets::spaced(&format!("{}  »  ", i.version), 12.0, t.muted, false);
                widgets::append(&mut job, &latest.unwrap_or_default(), 12.0, t.good, true);
                job
            }
            (Some(i), _) => widgets::spaced(&format!("v{}", i.version), 12.0, t.muted, false),
            (None, _) => match latest {
                Some(v) => widgets::spaced(&format!("v{v}"), 12.0, t.muted, false),
                None if e.checking => widgets::spaced("checking…", 12.0, t.faint, false),
                None => widgets::spaced("no release yet", 12.0, t.faint, false),
            },
        }
    }

    fn card(&mut self, ui: &mut Ui, i: usize, width: f32, actions: &mut Vec<Action>) {
        let t = self.t;
        let id = egui::Id::new(("card", &self.entries[i].app.id));
        let hovered_before = ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
        let fill = if hovered_before { t.card_hover } else { t.card };
        let [r, g, b] = self.entries[i].app.color;
        let app_color = Color32::from_rgb(r, g, b);
        let inner = egui::Frame::new().fill(fill).stroke(Stroke::new(1.0, t.border)).show(ui, |ui| {
            ui.set_width(width - 2.0);
            ui.set_min_height(self.card_h);
            ui.spacing_mut().item_spacing.y = 0.0;
            let top = ui.cursor().top();
            // Header strip: version and the GitHub link.
            egui::Frame::new().inner_margin(Margin::symmetric(22, 0)).show(ui, |ui| {
                ui.set_height(50.0);
                ui.horizontal_centered(|ui| {
                    let job = self.version_job(&self.entries[i]);
                    ui.label(job);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let r = self.github_link(ui);
                        if r.clicked() {
                            let e = &self.entries[i];
                            actions.push(Action::Url(e.app.id.clone(), e.app.github_url()));
                        }
                    });
                });
            });
            let line = ui.allocate_exact_size(vec2(width - 2.0, 1.0), Sense::hover()).0;
            ui.painter().rect_filled(line, 0.0, t.border);
            let body =
                egui::Frame::new().inner_margin(Margin { left: 22, right: 22, top: 22, bottom: 22 }).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    self.card_body(ui, i, actions);
                });
            self.next_card_h = self.next_card_h.max(body.response.rect.bottom() - top);
        });
        let rect = inner.response.rect;
        let hovered = ui.rect_contains_pointer(rect);
        if hovered != hovered_before {
            ui.ctx().data_mut(|d| d.insert_temp(id, hovered));
            ui.ctx().request_repaint();
        }
        if hovered {
            let top = egui::Rect::from_min_size(rect.min, vec2(rect.width(), 2.0));
            ui.painter().rect_filled(top, 0.0, app_color);
        }
    }

    fn github_link(&self, ui: &mut Ui) -> egui::Response {
        let t = self.t;
        let galley = ui.painter().layout_job(widgets::spaced("GitHub", 11.5, t.muted, false));
        let (rect, resp) = ui.allocate_exact_size(vec2(galley.size().x + 18.0, 24.0), Sense::click());
        let color = if resp.hovered() { t.text } else { t.muted };
        ui.painter().galley_with_override_text_color(
            pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
            galley,
            color,
        );
        let ir = egui::Rect::from_center_size(pos2(rect.right() - 5.0, rect.center().y), vec2(7.0, 7.0));
        widgets::paint_icon(ui, Icon::External, ir, color);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        resp.on_hover_text("Open the repository on GitHub")
    }

    fn by_keiko(&self, ui: &mut Ui) {
        let t = self.t;
        egui::Frame::new().fill(t.seal.gamma_multiply(0.16)).inner_margin(Margin::symmetric(8, 4)).show(ui, |ui| {
            ui.label(widgets::spaced("by Keiko", 10.0, t.seal, false));
        });
    }

    fn card_body(&mut self, ui: &mut Ui, i: usize, actions: &mut Vec<Action>) {
        let t = self.t;
        let content_w = ui.available_width();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            self.icon(ui, i, 72.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    ui.label(self.name_job(&self.entries[i], 29.0));
                    self.by_keiko(ui);
                });
                ui.add(
                    egui::Label::new(RichText::new(&self.entries[i].app.tagline).size(15.5).color(t.muted)).truncate(),
                );
            });
        });
        ui.add_space(22.0);
        let e = &self.entries[i];
        widgets::belt(ui, &t, e.state(), e.busy_label().as_deref());
        ui.add_space(18.0);

        ui.spacing_mut().item_spacing.y = 5.0;
        let size = e.latest.as_ref().and_then(|l| l.asset.as_ref()).map(|a| format::size(a.size));
        if let Some(s) = size {
            ui.label(RichText::new(format!("Download size: {s}")).size(14.0).color(t.text.gamma_multiply(0.85)));
        }
        ui.add(
            egui::Label::new(RichText::new(&e.app.description).size(14.0).color(t.text.gamma_multiply(0.85))).wrap(),
        );
        let released = e.latest.as_ref().and_then(|l| l.published_at.as_deref()).and_then(format::date);
        if let Some(d) = released {
            ui.label(RichText::new(format!("Released: {d}")).size(14.0).color(t.muted));
        }
        if let Some(err) = &e.error {
            ui.add(egui::Label::new(RichText::new(err).size(13.0).color(t.bad)).wrap());
        }
        ui.add_space(18.0);
        self.card_actions(ui, i, content_w, actions);
    }

    fn card_actions(&self, ui: &mut Ui, i: usize, width: f32, actions: &mut Vec<Action>) {
        let t = self.t;
        let e = &self.entries[i];
        let id = e.app.id.clone();
        if let Some(label) = e.busy_label() {
            let fraction = match e.busy {
                Some(Busy::Phase(Phase::Downloading { done, total })) if total > 0 => done as f32 / total as f32,
                Some(Busy::Phase(Phase::Verifying)) => 1.0,
                Some(Busy::Phase(Phase::Installing)) => 1.0,
                _ => 0.0,
            };
            let (rect, _) = ui.allocate_exact_size(vec2(width, 40.0), Sense::hover());
            let p = ui.painter();
            p.rect_stroke(rect, 2.0, Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
            let mut fill = rect.shrink(1.0);
            fill.set_width(fill.width() * fraction);
            p.rect_filled(fill, 1.0, t.seal.gamma_multiply(0.28));
            let g = ui.painter().layout_job(widgets::spaced(&label, 12.0, t.text, true));
            ui.painter().galley(rect.center() - g.size() / 2.0, g, t.text);
            return;
        }
        let gap = 8.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            let rest = width - MORE_W - gap;
            match e.state() {
                State::UpdateAvailable => {
                    let half = ((rest - gap) / 2.0).floor();
                    let installed = e.installed.as_ref().map(|i| i.version.clone()).unwrap_or_default();
                    let latest = e.latest.as_ref().map(|l| l.version.clone()).unwrap_or_default();
                    if widgets::button(ui, &t, &format!("Open {installed}"), Kind::Secondary, half, true).clicked() {
                        actions.push(Action::Open(id.clone()));
                    }
                    if widgets::button(ui, &t, &format!("Update to {latest}"), Kind::Primary, half, true).clicked() {
                        actions.push(Action::Install(id.clone()));
                    }
                }
                State::UpToDate | State::Installed => {
                    if widgets::button(ui, &t, "Open", Kind::Primary, rest, true).clicked() {
                        actions.push(Action::Open(id.clone()));
                    }
                }
                State::NotInstalled => {
                    if widgets::button(ui, &t, "Install", Kind::Primary, rest, true).clicked() {
                        actions.push(Action::Install(id.clone()));
                    }
                }
                State::Unavailable => {
                    let label = if e.checking { "Checking…" } else { "Not available yet" };
                    widgets::button(ui, &t, label, Kind::Secondary, rest, false);
                }
            }
            self.more_menu(ui, i, actions);
        });
    }

    fn more_menu(&self, ui: &mut Ui, i: usize, actions: &mut Vec<Action>) {
        let t = self.t;
        let e = &self.entries[i];
        let id = e.app.id.clone();
        let resp = widgets::icon_button(ui, &t, Icon::More, false, "More");
        egui::Popup::menu(&resp).width(210.0).show(|ui| {
            ui.spacing_mut().button_padding = vec2(10.0, 7.0);
            if let Some(l) = &e.latest
                && !l.html_url.is_empty()
                && ui.button(RichText::new(format!("Release notes ({})", l.tag)).size(14.0)).clicked()
            {
                actions.push(Action::Url(id.clone(), l.html_url.clone()));
            }
            if ui.button(RichText::new("View on GitHub").size(14.0)).clicked() {
                actions.push(Action::Url(id.clone(), e.app.github_url()));
            }
            if e.installed.is_some() && ui.button(RichText::new("Show in Finder").size(14.0)).clicked() {
                actions.push(Action::Reveal(id.clone()));
            }
            if ui.button(RichText::new("Check again").size(14.0)).clicked() {
                actions.push(Action::Check(id.clone()));
            }
            if e.installed.is_some() {
                ui.separator();
                if ui.button(RichText::new("Move to Trash…").size(14.0).color(t.bad)).clicked() {
                    actions.push(Action::AskUninstall(id.clone()));
                }
            }
        });
    }

    fn list(&mut self, ui: &mut Ui, group: &[usize], actions: &mut Vec<Action>) {
        let t = self.t;
        let width = ui.available_width();
        let cols = [64.0, 0.0, 240.0, 110.0, 110.0, 140.0, 0.0];
        let fixed: f32 = cols.iter().sum::<f32>() + 260.0;
        let name_w = (width - fixed).max(220.0);
        egui::Frame::new().stroke(Stroke::new(1.0, t.border)).show(ui, |ui| {
            ui.set_width(width - 2.0);
            ui.spacing_mut().item_spacing.y = 0.0;
            egui::Frame::new().fill(t.panel).inner_margin(Margin::symmetric(16, 10)).show(ui, |ui| {
                ui.set_width(width - 34.0);
                ui.horizontal(|ui| {
                    for (text, w) in [
                        ("", cols[0]),
                        ("Dojo", name_w),
                        ("Status", cols[2]),
                        ("Installed", cols[3]),
                        ("Latest", cols[4]),
                        ("Released", cols[5]),
                    ] {
                        ui.allocate_ui_with_layout(
                            vec2(w, 18.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.set_width(w);
                                widgets::label(ui, text, 11.0, t.muted);
                            },
                        );
                    }
                });
            });
            for i in group {
                let line = ui.allocate_exact_size(vec2(width - 2.0, 1.0), Sense::hover()).0;
                ui.painter().rect_filled(line, 0.0, t.border);
                egui::Frame::new().inner_margin(Margin::symmetric(16, 12)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.allocate_ui_with_layout(
                            vec2(cols[0], 44.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.set_width(cols[0]);
                                self.icon(ui, *i, 44.0);
                            },
                        );
                        let e = &self.entries[*i];
                        ui.allocate_ui_with_layout(
                            vec2(name_w, 44.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(name_w);
                                ui.spacing_mut().item_spacing.y = 2.0;
                                ui.label(self.name_job(e, 19.0));
                                ui.add(
                                    egui::Label::new(RichText::new(&e.app.tagline).size(13.5).color(t.muted))
                                        .truncate(),
                                );
                            },
                        );
                        ui.allocate_ui_with_layout(
                            vec2(cols[2], 44.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.set_width(cols[2]);
                                widgets::belt(ui, &t, e.state(), e.busy_label().as_deref());
                            },
                        );
                        let mono = |s: String, c: Color32| RichText::new(s).family(theme::mono()).size(13.0).color(c);
                        let installed = e.installed.as_ref().map(|i| i.version.clone()).unwrap_or_else(|| "—".into());
                        let latest = e.latest.as_ref().map(|l| l.version.clone()).unwrap_or_else(|| "—".into());
                        let released = e
                            .latest
                            .as_ref()
                            .and_then(|l| l.published_at.as_deref())
                            .and_then(format::date)
                            .unwrap_or_else(|| "—".into());
                        let latest_color = if e.state() == State::UpdateAvailable { t.good } else { t.text };
                        for (text, w, c) in [
                            (installed, cols[3], t.text),
                            (latest, cols[4], latest_color),
                            (released, cols[5], t.muted),
                        ] {
                            ui.allocate_ui_with_layout(
                                vec2(w, 44.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.set_width(w);
                                    ui.label(mono(text, c));
                                },
                            );
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            self.more_menu(ui, *i, actions);
                            let e = &self.entries[*i];
                            let id = e.app.id.clone();
                            match (e.busy.is_some(), e.state()) {
                                (true, _) => {}
                                (_, State::UpdateAvailable) => {
                                    if widgets::button(ui, &t, "Update", Kind::Primary, 120.0, true).clicked() {
                                        actions.push(Action::Install(id));
                                    }
                                }
                                (_, State::UpToDate | State::Installed) => {
                                    if widgets::button(ui, &t, "Open", Kind::Secondary, 120.0, true).clicked() {
                                        actions.push(Action::Open(id));
                                    }
                                }
                                (_, State::NotInstalled) => {
                                    if widgets::button(ui, &t, "Install", Kind::Primary, 120.0, true).clicked() {
                                        actions.push(Action::Install(id));
                                    }
                                }
                                (_, State::Unavailable) => {}
                            }
                        });
                    });
                });
            }
        });
    }

    // --- overlays -----------------------------------------------------------------------------

    fn confirm(&mut self, ctx: &egui::Context) {
        let Some(id) = self.confirm_uninstall.clone() else { return };
        let Some(i) = self.entries.iter().position(|e| e.app.id == id) else {
            self.confirm_uninstall = None;
            return;
        };
        let t = self.t;
        let name = self.entries[i].app.name.clone();
        let mut choice = None;
        let modal = egui::Modal::new(egui::Id::new("confirm-uninstall")).show(ctx, |ui| {
            ui.set_width(380.0);
            ui.label(RichText::new(format!("Move {name} to the Trash?")).size(20.0).family(theme::display()));
            ui.add_space(8.0);
            ui.label(
                RichText::new("The app goes to the Trash, where you can restore it. Its data stays on this Mac.")
                    .size(14.5)
                    .color(t.muted),
            );
            ui.add_space(18.0);
            ui.horizontal(|ui| {
                if widgets::button(ui, &t, "Move to Trash", Kind::Primary, 0.0, true).clicked() {
                    choice = Some(true);
                }
                if widgets::button(ui, &t, "Cancel", Kind::Secondary, 0.0, true).clicked() {
                    choice = Some(false);
                }
            });
        });
        if modal.should_close() && choice.is_none() {
            choice = Some(false);
        }
        if let Some(go) = choice {
            self.confirm_uninstall = None;
            let e = &mut self.entries[i];
            if go && let Some(installed) = e.installed.clone() {
                e.busy = Some(Busy::Uninstalling);
                self.jobs.uninstall(&e.app, &installed);
            }
        }
    }

    fn toasts(&mut self, ctx: &egui::Context) {
        self.toasts.retain(|(_, _, at)| at.elapsed() < Duration::from_secs(6));
        if self.toasts.is_empty() {
            return;
        }
        let t = self.t;
        egui::Area::new(egui::Id::new("toasts"))
            .anchor(egui::Align2::RIGHT_BOTTOM, vec2(-24.0, -54.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                for (text, ok, _) in &self.toasts {
                    egui::Frame::new()
                        .fill(t.card)
                        .stroke(Stroke::new(1.0, if *ok { t.border_strong } else { t.bad }))
                        .inner_margin(Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.set_max_width(420.0);
                            ui.label(RichText::new(text).size(14.0).color(if *ok { t.text } else { t.bad }));
                        });
                    ui.add_space(8.0);
                }
            });
        ctx.request_repaint_after(Duration::from_millis(500));
    }

    fn screenshot(&mut self, ctx: &egui::Context) {
        let Some((path, since, requested)) = self.shot.clone() else { return };
        let image = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = image {
            let [w, h] = image.size;
            let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
            if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, bytes)
                && let Err(e) = buf.save(&path)
            {
                eprintln!("screenshot: {e}");
            }
            self.shot = None;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let settled = since.elapsed() > Duration::from_secs(2) && self.pending == 0;
        if !requested && (settled || since.elapsed() > Duration::from_secs(25)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            self.shot = Some((path, since, true));
        }
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
