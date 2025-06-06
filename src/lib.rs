use eframe::egui;
use glam::{Quat, Vec3};

#[derive(Default)]
pub struct CsgrsApp {
    rotation: Quat,
    translation: egui::Vec2,
    zoom: f32,
}

impl CsgrsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            rotation: Quat::IDENTITY,
            translation: egui::Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl eframe::App for CsgrsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_min_size(ui.available_size());
            let (rect, response) =
                ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

            // ───── Interaction ─────
            if response.dragged() {
                let delta = response.drag_delta();
                let input = ui.input(|i| i.clone());
                if input.pointer.primary_down() {
                    // left‑drag → rotate
                    let yaw = delta.x * 0.01;
                    let pitch = delta.y * 0.01;
                    self.rotation =
                        Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch) * self.rotation;
                } else if input.pointer.secondary_down() {
                    // right‑drag → pan
                    self.translation += delta;
                }
            }

            // scroll → zoom
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll.abs() > 0.0 {
                self.zoom = (self.zoom * (1.0 + scroll * 0.001)).clamp(0.2, 5.0);
            }

            // ───── Paint ─────
            let painter = ui.painter_at(rect);
            draw_cube(&painter, rect, self);
        });
    }
}

// ───── Geometry ─────
const VERTICES: [Vec3; 8] = [
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, 1.0),
];

const EDGES: [(usize, usize); 12] = [
    (0, 1), (1, 2), (2, 3), (3, 0),
    (4, 5), (5, 6), (6, 7), (7, 4),
    (0, 4), (1, 5), (2, 6), (3, 7),
];

fn draw_cube(painter: &egui::Painter, rect: egui::Rect, app: &CsgrsApp) {
    let stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
    let size = rect.width().min(rect.height()) * 0.25 * app.zoom;

    // project vertices to screen space
    let mut projected = Vec::<egui::Pos2>::with_capacity(8);
    for &v in &VERTICES {
        let rotated = app.rotation * v;
        // basic perspective projection
        let dist = 4.0;
        let scale = dist / (dist - rotated.z);
        let p = egui::vec2(rotated.x * scale, rotated.y * scale);

        let offset = (egui::vec2(p.x, -p.y) * size) + app.translation;
        let screen = rect.center() + offset;
        projected.push(screen);
    }

    // draw edges
    for &(a, b) in &EDGES {
        painter.line_segment([projected[a], projected[b]], stroke);
    }
}

// ── Web entry‑point ──
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    // Redirect `log` macros & panic messages to the browser console
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    console_error_panic_hook::set_once();

    let web_options = eframe::WebOptions::default();

    // The element id must match the <canvas> in your index.html
    eframe::WebRunner::new()
        .start(
            "csgrs_canvas", // canvas id
            web_options,
            Box::new(|cc| Box::new(CsgrsApp::new(cc))),
        )
        .await?;

    Ok(())
}

// ── Native entry‑point ──
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "csgrs egui wasm example",
        options,
        Box::new(|cc| Box::new(CsgrsApp::new(cc))),
    )
}

