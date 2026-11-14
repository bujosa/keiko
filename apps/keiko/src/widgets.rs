//! Keiko's controls: letter-spaced mono labels, ink buttons, belt badges and painted icons.

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, CornerRadius, FontId, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use keiko_core::State;

use crate::theme::{self, Tokens};

/// Uppercase mono text with tracking, like a printed label.
pub fn spaced(text: &str, size: f32, color: Color32, bold: bool) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, text, size, color, bold);
    job
}

/// Add a spaced label run to `job` (several colours in one label).
pub fn append(job: &mut LayoutJob, text: &str, size: f32, color: Color32, bold: bool) {
    let family = if bold { theme::mono_bold() } else { theme::mono() };
    job.append(
        &text.to_uppercase(),
        0.0,
        TextFormat {
            font_id: FontId::new(size, family),
            color,
            extra_letter_spacing: size * 0.16,
            ..Default::default()
        },
    );
}

pub fn label(ui: &mut Ui, text: &str, size: f32, color: Color32) -> Response {
    ui.label(spaced(text, size, color, false))
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Primary,
    Secondary,
}

/// An ink button with a mono label. `width` 0 sizes it to the label.
pub fn button(ui: &mut Ui, t: &Tokens, text: &str, kind: Kind, width: f32, enabled: bool) -> Response {
    let (fg, bg, stroke) = match kind {
        Kind::Primary => (t.primary_text, t.primary_bg, t.primary_bg),
        Kind::Secondary => (t.text, Color32::TRANSPARENT, t.border_strong),
    };
    let galley = ui.painter().layout_job(spaced(text, 12.0, fg, true));
    let w = if width > 0.0 { width } else { galley.size().x + 36.0 };
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 40.0), sense);
    let hovered = enabled && resp.hovered();
    let bg = match (kind, hovered) {
        (Kind::Primary, true) => bg.gamma_multiply(0.86),
        (Kind::Secondary, true) => t.card_hover,
        _ => bg,
    };
    let alpha = if enabled { 1.0 } else { 0.4 };
    let p = ui.painter();
    p.rect(
        rect,
        CornerRadius::same(2),
        bg.gamma_multiply(alpha),
        Stroke::new(1.0, stroke.gamma_multiply(alpha)),
        StrokeKind::Inside,
    );
    let pos = rect.center() - galley.size() / 2.0;
    p.galley_with_override_text_color(pos, galley, fg.gamma_multiply(alpha));
    focus_ring(ui, &resp, rect);
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn focus_ring(ui: &Ui, resp: &Response, rect: Rect) {
    if resp.has_focus() {
        let c = ui.visuals().selection.stroke.color;
        ui.painter().rect_stroke(rect.expand(2.0), 3.0, Stroke::new(2.0, c), StrokeKind::Outside);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Icon {
    Refresh,
    Grid,
    List,
    More,
    External,
    Theme,
}

/// A square icon button (`active` draws it pressed, like the view toggle).
pub fn icon_button(ui: &mut Ui, t: &Tokens, icon: Icon, active: bool, hint: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::click());
    let (bg, fg) = if active {
        (t.primary_bg, t.primary_text)
    } else if resp.hovered() {
        (t.card_hover, t.text)
    } else {
        (Color32::TRANSPARENT, t.text)
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(2),
        bg,
        Stroke::new(1.0, if active { bg } else { t.border_strong }),
        StrokeKind::Inside,
    );
    paint_icon(ui, icon, rect.shrink(12.0), fg);
    focus_ring(ui, &resp, rect);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.on_hover_text(hint)
}

pub fn paint_icon(ui: &Ui, icon: Icon, r: Rect, c: Color32) {
    let p = ui.painter();
    let s = Stroke::new(1.6, c);
    match icon {
        Icon::Refresh => {
            let center = r.center();
            let rad = r.width() * 0.45;
            let pts: Vec<_> = (0..=28)
                .map(|i| {
                    let a = -1.2 + i as f32 / 28.0 * 5.0;
                    center + vec2(a.cos(), a.sin()) * rad
                })
                .collect();
            let tip = pts[0];
            p.add(egui::Shape::line(pts, s));
            p.line_segment([tip, tip + vec2(-5.0, -1.0)], s);
            p.line_segment([tip, tip + vec2(0.5, -5.0)], s);
        }
        Icon::Grid => {
            let g = 2.0;
            let w = (r.width() - g) / 2.0;
            for (x, y) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
                let min = r.min + vec2(x * (w + g), y * (w + g));
                p.rect_filled(Rect::from_min_size(min, Vec2::splat(w)), 1.0, c);
            }
        }
        Icon::List => {
            for i in 0..3 {
                let y = r.top() + 2.0 + i as f32 * (r.height() - 4.0) / 2.0;
                p.line_segment([pos2(r.left(), y), pos2(r.right(), y)], Stroke::new(2.0, c));
            }
        }
        Icon::More => {
            for i in -1..=1 {
                p.circle_filled(r.center() + vec2(i as f32 * 5.0, 0.0), 1.6, c);
            }
        }
        Icon::External => {
            let (a, b) = (pos2(r.left(), r.bottom()), pos2(r.right(), r.top()));
            p.line_segment([a, b], s);
            p.line_segment([b, b + vec2(-r.width() * 0.6, 0.0)], s);
            p.line_segment([b, b + vec2(0.0, r.height() * 0.6)], s);
        }
        Icon::Theme => {
            let rad = r.width() * 0.48;
            p.circle_stroke(r.center(), rad, s);
            let half: Vec<_> = (0..=20)
                .map(|i| {
                    let a = -std::f32::consts::FRAC_PI_2 + i as f32 / 20.0 * std::f32::consts::PI;
                    r.center() + vec2(a.cos(), a.sin()) * rad
                })
                .collect();
            p.add(egui::Shape::convex_polygon(half, c, Stroke::NONE));
        }
    }
}

/// The belt an app wears: a coloured swatch and what it means.
pub fn belt(ui: &mut Ui, t: &Tokens, state: State, busy: Option<&str>) -> Response {
    let (swatch, text, color, hint) = match (busy, state) {
        (Some(b), _) => (t.seal, b.to_string(), t.seal, "Working on it"),
        (None, State::UpdateAvailable) => {
            (t.amber, "Update available".into(), t.amber, "Yellow belt: a newer version is out")
        }
        (None, State::UpToDate) => (
            if t.dark { Color32::from_rgb(6, 6, 8) } else { Color32::from_rgb(22, 23, 27) },
            "Up to date".into(),
            t.good,
            "Black belt: you have the latest version",
        ),
        (None, State::Installed) => {
            (t.muted, "Installed".into(), t.muted, "Installed; the latest release is not known yet")
        }
        (None, State::NotInstalled) => {
            (Color32::from_rgb(246, 243, 236), "Not installed".into(), t.muted, "White belt: ready to install")
        }
        (None, State::Unavailable) => {
            (t.faint, "No build for this Mac".into(), t.faint, "The latest release has no build for this Mac")
        }
    };
    let galley = ui.painter().layout_job(spaced(&text, 11.0, color, false));
    let size = vec2(galley.size().x + 48.0, 28.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter();
    p.rect_stroke(rect, CornerRadius::same(2), Stroke::new(1.0, color.gamma_multiply(0.55)), StrokeKind::Inside);
    let sw = Rect::from_min_size(pos2(rect.left() + 9.0, rect.center().y - 6.0), vec2(22.0, 12.0));
    knot(p, sw, swatch, t.border_strong);
    p.galley(pos2(sw.right() + 8.0, rect.center().y - galley.size().y / 2.0), galley, color);
    resp.on_hover_text(hint)
}

/// A thin bar.
pub fn bar(ui: &mut Ui, t: &Tokens, fraction: f32, width: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 4.0), Sense::hover());
    let id = ui.id().with(("bar", rect.min.x as i32, rect.min.y as i32));
    let f = ui.ctx().animate_value_with_time(id, fraction.clamp(0.0, 1.0), 0.3);
    let p = ui.painter();
    p.rect_filled(rect, 0.0, t.border);
    let mut fill = rect;
    fill.set_width(rect.width() * f);
    if fill.width() > 0.5 {
        p.rect_filled(fill, 0.0, color);
    }
}

/// A fallback app tile: the app's colour with its initial.
pub fn initial_tile(ui: &mut Ui, color: Color32, name: &str, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    ui.painter().rect_filled(rect, size * 0.22, color);
    let letter = name.chars().next().unwrap_or('?').to_string();
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        letter,
        FontId::new(size * 0.5, theme::display()),
        Color32::BLACK,
    );
}

/// A tiny knotted belt in `fill` (the status badge's swatch).
fn knot(p: &egui::Painter, r: Rect, fill: Color32, edge: Color32) {
    let s = Stroke::new(1.0, edge);
    let band = Rect::from_min_size(pos2(r.left(), r.top() + 2.0), vec2(r.width(), 4.0));
    p.rect(band, 0.5, fill, s, StrokeKind::Inside);
    let cx = r.center().x;
    let tail = |dx: f32| {
        vec![
            pos2(cx - 1.5, r.top() + 5.0),
            pos2(cx + 1.5, r.top() + 5.0),
            pos2(cx + dx + 1.5, r.bottom()),
            pos2(cx + dx - 1.5, r.bottom()),
        ]
    };
    p.add(egui::Shape::convex_polygon(tail(-4.0), fill, s));
    p.add(egui::Shape::convex_polygon(tail(4.0), fill, s));
    p.rect(Rect::from_center_size(pos2(cx, r.top() + 4.0), vec2(5.0, 7.0)), 1.0, fill, s, StrokeKind::Inside);
}

/// A section heading: a spaced label and a hairline to the right edge.
pub fn section_title(ui: &mut Ui, t: &Tokens, text: &str) {
    ui.horizontal(|ui| {
        label(ui, text, 12.0, t.muted);
        let r = ui.available_rect_before_wrap();
        let y = ui.min_rect().center().y;
        ui.painter().line_segment([pos2(r.left() + 10.0, y), pos2(r.right(), y)], Stroke::new(1.0, t.border));
    });
}

/// A progress ring with the percentage in the middle.
pub fn ring(ui: &mut Ui, t: &Tokens, fraction: f32, size: f32, color: Color32, unit: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let id = ui.id().with("ring");
    let f = ui.ctx().animate_value_with_time(id, fraction.clamp(0.0, 1.0), 0.6);
    let c = rect.center();
    let rad = size / 2.0 - 6.0;
    let p = ui.painter();
    p.circle_stroke(c, rad, Stroke::new(7.0, t.border));
    if f > 0.001 {
        let steps = (96.0 * f).ceil().max(2.0) as usize;
        let pts: Vec<_> = (0..=steps)
            .map(|i| {
                let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * f * i as f32 / steps as f32;
                c + vec2(a.cos(), a.sin()) * rad
            })
            .collect();
        p.add(egui::Shape::line(pts, Stroke::new(7.0, color)));
    }
    // Where the ring starts, so 0% still points the way.
    p.circle_filled(c - vec2(0.0, rad), 5.0, color);
    let pct = format!("{}%", (f * 100.0).round() as u32);
    p.text(c - vec2(0.0, 8.0), egui::Align2::CENTER_CENTER, pct, FontId::new(size * 0.22, theme::display()), t.text);
    let g = p.layout_job(spaced(unit, 10.5, t.muted, false));
    p.galley(pos2(c.x - g.size().x / 2.0, c.y + size * 0.1), g, t.muted);
}

/// A dashed rectangle outline.
pub fn dashed_rect(r: Rect, stroke: Stroke) -> Vec<egui::Shape> {
    let pts = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
    egui::Shape::dashed_line(&pts, stroke, 6.0, 5.0)
}

/// A large, faint training belt on the right of a panel, fading in from the left.
pub fn belt_motif(r: Rect, t: &Tokens) -> egui::Shape {
    let ink = if t.dark { Color32::from_white_alpha(12) } else { Color32::from_black_alpha(12) };
    let y = r.center().y + 6.0;
    let kx = r.left() + r.width() * 0.8;
    let start = r.left() + r.width() * 0.5;
    let mut mesh = egui::Mesh::default();
    for (x, c) in [(start, Color32::TRANSPARENT), (r.right(), ink)] {
        mesh.colored_vertex(pos2(x, y - 24.0), c);
        mesh.colored_vertex(pos2(x, y + 24.0), c);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);
    let mut shapes = vec![egui::Shape::mesh(mesh)];
    let tail = |dx: f32| {
        egui::Shape::convex_polygon(
            vec![
                pos2(kx - 18.0, y + 20.0),
                pos2(kx + 18.0, y + 20.0),
                pos2(kx + dx + 16.0, r.bottom()),
                pos2(kx + dx - 16.0, r.bottom()),
            ],
            ink,
            Stroke::NONE,
        )
    };
    shapes.push(tail(-50.0));
    shapes.push(tail(50.0));
    let knot_rect = Rect::from_center_size(pos2(kx, y), vec2(70.0, 82.0));
    shapes.push(egui::Shape::rect_filled(knot_rect, 12.0, ink));
    shapes.push(egui::Shape::rect_stroke(
        knot_rect,
        12.0,
        Stroke::new(1.5, t.seal.gamma_multiply(0.45)),
        StrokeKind::Inside,
    ));
    egui::Shape::Vec(shapes)
}

/// Keiko's mark for small sizes: the vermilion seal with a knotted belt.
pub fn mark(ui: &mut Ui, t: &Tokens, size: f32) {
    let (r, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, size * 0.22, t.seal);
    let paper = Color32::from_rgb(252, 248, 239);
    let u = size / 30.0;
    let y = r.top() + 12.5 * u;
    p.rect_filled(
        Rect::from_x_y_ranges(r.left() + 4.0 * u..=r.right() - 4.0 * u, y - 2.5 * u..=y + 2.5 * u),
        0.5,
        paper,
    );
    let cx = r.center().x;
    let tail = |dx: f32| {
        egui::Shape::convex_polygon(
            vec![
                pos2(cx - 2.0 * u, y + 2.0 * u),
                pos2(cx + 2.0 * u, y + 2.0 * u),
                pos2(cx + dx + 2.2 * u, r.bottom() - 5.0 * u),
                pos2(cx + dx - 2.2 * u, r.bottom() - 5.0 * u),
            ],
            paper,
            Stroke::NONE,
        )
    };
    p.add(tail(-5.0 * u));
    p.add(tail(5.0 * u));
    let knot = Rect::from_center_size(pos2(cx, y), vec2(7.0 * u, 9.0 * u));
    p.rect(knot, 1.5 * u, paper, Stroke::new(1.2 * u, t.seal), StrokeKind::Outside);
}
