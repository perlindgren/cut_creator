#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 500.0)),
        ..Default::default()
    };
    eframe::run_native("Splines", options, Box::new(|_cc| Box::<MyApp>::default()))
}

#[derive(Default)]
struct MyApp {
    bezier: Splines,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("CatmullRom");

            //
            Frame::canvas(ui.style()).show(ui, |ui| {
                self.bezier.ui_content(ui);
            });
        });
    }
}

use egui::epaint::PathShape;
use egui::*;

pub struct Splines {
    /// The control points. The [`Self::degree`] first of them are used.
    control_points: Vec<Pos2>,

    /// Selected
    control_selected: Vec<bool>,

    /// Stroke selected.
    stroke_default: Stroke,

    /// Stroke selected.
    stroke_selected: Stroke,

    /// Stroke for auxiliary lines.
    line_stroke: Stroke,
}

impl Default for Splines {
    fn default() -> Self {
        Self {
            control_points: vec![
                pos2(0.0, 0.0),
                pos2(60.0, 250.0),
                pos2(200.0, 200.0),
                pos2(250.0, 50.0),
            ],
            control_selected: vec![false; 4],
            stroke_default: Stroke::new(1.0, Color32::WHITE.linear_multiply(0.5)),
            stroke_selected: Stroke::new(1.0, Color32::WHITE),

            line_stroke: Stroke::new(1.0, Color32::RED.linear_multiply(0.25)),
        }
    }
}

impl Splines {
    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), ui.available_height()),
            Sense::click_and_drag(),
        );

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let mut clicked = response.clicked();

        let control_point_radius = 8.0;

        if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
            println!("delete");
            let cp = self
                .control_points
                .clone()
                .into_iter()
                .zip(self.control_selected.clone().into_iter());

            (self.control_points, self.control_selected) =
                cp.filter(|(_, selected)| !*selected).unzip();
        }

        let cp = self.control_points.clone();
        let control_point_shapes: Vec<Shape> = self
            .control_points
            .iter_mut()
            .enumerate()
            .map(|(i, point)| {
                let size = Vec2::splat(2.0 * control_point_radius);

                let point_in_screen = to_screen.transform_pos(*point);
                let point_rect = Rect::from_center_size(point_in_screen, size);

                let point_id = response.id.with(i);
                let point_click = ui.interact(point_rect, point_id, Sense::click());
                if point_click.clicked() {
                    self.control_selected[i] = !self.control_selected[i];
                    clicked = false;
                }

                let min_x = if i > 0 { cp.get(i - 1).unwrap().x } else { 0.0 };
                let max_x = cp.get(i + 1).unwrap_or(&response.rect.max).x;

                let point_response = ui.interact(point_rect, point_id, Sense::drag());

                let delta = point_response.drag_delta();

                if delta != Vec2::ZERO {
                    *point += point_response.drag_delta();
                    *point = to_screen.from().clamp(*point);
                    point.x = point.x.min(max_x).max(min_x);
                }

                let point_in_screen = to_screen.transform_pos(*point);

                Shape::circle_stroke(
                    point_in_screen,
                    control_point_radius,
                    if self.control_selected[i] {
                        self.stroke_selected
                    } else {
                        self.stroke_default
                    },
                )
            })
            .collect();

        if clicked {
            let pos = response.interact_pointer_pos().unwrap();
            println!(
                "clicked outside {:?}, {:?}, {:?}",
                pos,
                to_screen.inverse().transform_pos(pos),
                to_screen.inverse().transform_pos_clamped(pos)
            );

            let pos = to_screen.inverse().transform_pos_clamped(pos);

            // insert
            let cp = self
                .control_points
                .clone()
                .into_iter()
                .zip(self.control_selected.clone().into_iter());

            let (head, mut tail): (Vec<_>, Vec<_>) = cp.partition(|(p2, _)| pos.x < p2.x);

            tail.push((pos, false));
            tail.extend(head);

            (self.control_points, self.control_selected) = tail.into_iter().unzip();
        }

        let points_in_screen: Vec<Pos2> =
            self.control_points.iter().map(|p| to_screen * *p).collect();

        painter.add(PathShape::line(points_in_screen, self.line_stroke));
        painter.extend(control_point_shapes);

        response
    }
}
