//! Work off the UI thread. Each job sends messages back and wakes the window.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

use keiko_core::{App, Latest};
use keiko_native::install::Phase;
use keiko_native::{Installed, Paths, detect, github, install};

pub enum Msg {
    Detected(String, Option<Installed>),
    Latest(String, Result<Option<Latest>, String>),
    Phase(String, Phase),
    Installed(String, Result<Installed, String>),
    Uninstalled(String, Result<PathBuf, String>),
    Failed(String, String),
}

pub struct Jobs {
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    ctx: egui::Context,
}

impl Jobs {
    pub fn new(ctx: &egui::Context) -> Jobs {
        let (tx, rx) = channel();
        Jobs { tx, rx, ctx: ctx.clone() }
    }

    pub fn poll(&self) -> Vec<Msg> {
        self.rx.try_iter().collect()
    }

    fn spawn(&self, work: impl FnOnce(&dyn Fn(Msg)) + Send + 'static) {
        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            let send = |m: Msg| {
                let _ = tx.send(m);
                ctx.request_repaint();
            };
            work(&send);
        });
    }

    /// Find the installed copy, then ask GitHub for the latest release.
    pub fn check(&self, paths: &Paths, app: &App) {
        let (paths, app) = (paths.clone(), app.clone());
        self.spawn(move |send| {
            send(Msg::Detected(app.id.clone(), detect::installed(&paths, &app)));
            send(Msg::Latest(app.id.clone(), github::latest(&app)));
        });
    }

    /// Keiko itself: the running bundle is the installed copy, so only GitHub is asked.
    pub fn check_latest(&self, app: &App) {
        let app = app.clone();
        self.spawn(move |send| send(Msg::Latest(app.id.clone(), github::latest(&app))));
    }

    /// Only the local side (when the window comes back to the front).
    pub fn local(&self, paths: &Paths, app: &App) {
        let (paths, app) = (paths.clone(), app.clone());
        self.spawn(move |send| send(Msg::Detected(app.id.clone(), detect::installed(&paths, &app))));
    }

    pub fn install(&self, paths: &Paths, app: &App, latest: &Latest, current: Option<&Installed>) {
        let (paths, app, latest, current) = (paths.clone(), app.clone(), latest.clone(), current.cloned());
        self.spawn(move |send| {
            let id = app.id.clone();
            let result =
                install::install(&paths, &app, &latest, current.as_ref(), &|p| send(Msg::Phase(id.clone(), p)));
            send(Msg::Installed(id, result));
        });
    }

    pub fn uninstall(&self, paths: &Paths, app: &App, installed: &Installed) {
        let (paths, app, installed) = (paths.clone(), app.clone(), installed.clone());
        self.spawn(move |send| send(Msg::Uninstalled(app.id.clone(), install::uninstall(&paths, &app, &installed))));
    }

    /// Run a quick system action (open, reveal, browse, notify) and report only failures.
    pub fn act(&self, id: &str, action: impl FnOnce() -> Result<(), String> + Send + 'static) {
        let id = id.to_string();
        self.spawn(move |send| {
            if let Err(e) = action() {
                send(Msg::Failed(id, e));
            }
        });
    }
}
