//! Keiko: one place to install, update and open the dojo apps. A native egui window (glow
//! renderer, no web view).

mod app;
mod jobs;
mod theme;
mod widgets;

struct Shell(app::KeikoApp);

impl eframe::App for Shell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.frame(ui);
    }
}

fn main() -> eframe::Result {
    // Development aid for screenshots: `KEIKO_WINDOW=1400x900`.
    let size = std::env::var("KEIKO_WINDOW")
        .ok()
        .and_then(|s| {
            let (w, h) = s.split_once('x')?;
            Some([w.trim().parse::<f32>().ok()?, h.trim().parse::<f32>().ok()?])
        })
        .unwrap_or([1280.0, 860.0]);
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Keiko")
        .with_app_id("dev.bujosa.keiko")
        .with_inner_size(size)
        .with_min_inner_size([760.0, 560.0])
        .with_fullsize_content_view(true)
        .with_titlebar_shown(false)
        .with_title_shown(false);
    if let Some((rgba, width, height)) = app::icon_rgba() {
        viewport = viewport.with_icon(egui::IconData { rgba, width, height });
    }
    let options = eframe::NativeOptions { viewport, renderer: eframe::Renderer::Glow, ..Default::default() };
    eframe::run_native("Keiko", options, Box::new(|cc| Ok(Box::new(Shell(app::KeikoApp::new(&cc.egui_ctx))))))
}
