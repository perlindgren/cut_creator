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
use splines::{Interpolation, Key, Spline};

pub struct Splines {
    /// The control points.
    knots: Vec<Pos2>,

    /// Selected
    knots_selected: Vec<bool>,

    /// Stroke selected.
    stroke_default: Stroke,

    /// Stroke selected.
    stroke_selected: Stroke,

    /// Stroke for auxiliary lines.
    line_stroke: Stroke,

    /// Stroke for splines.
    spline_stroke: Stroke,

    /// Spline
    spline: Spline<f32, f32>,
}

impl Splines {
    // call to update spline when knots are changed
    fn update(&mut self) {
        self.spline = Spline::from_iter(
            self.knots
                .iter()
                .map(|p| Key::new(p[0], p[1], Interpolation::CatmullRom)),
        );
    }
}

impl Default for Splines {
    fn default() -> Self {
        let knots = vec![
            pos2(0.0, 0.0),
            pos2(60.0, 250.0),
            pos2(200.0, 200.0),
            pos2(250.0, 50.0),
        ];
        let spline = Spline::from_iter(
            knots
                .iter()
                .map(|p| Key::new(p[0], p[1], Interpolation::CatmullRom)),
        );

        Self {
            knots,
            knots_selected: vec![false; 4],
            stroke_default: Stroke::new(1.0, Color32::WHITE.linear_multiply(0.25)),
            stroke_selected: Stroke::new(1.0, Color32::WHITE),
            line_stroke: Stroke::new(1.0, Color32::RED.linear_multiply(0.25)),
            spline_stroke: Stroke::new(1.0, Color32::BLUE.linear_multiply(1.0)),
            spline,
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
        let mut update = false;

        let control_point_radius = 8.0;

        if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
            println!("delete");
            let cp = self
                .knots
                .clone()
                .into_iter()
                .zip(self.knots_selected.clone().into_iter());

            (self.knots, self.knots_selected) = cp.filter(|(_, selected)| !*selected).unzip();

            update = true;
        }

        let cp = self.knots.clone();
        let control_point_shapes: Vec<Shape> = self
            .knots
            .iter_mut()
            .enumerate()
            .map(|(i, point)| {
                let size = Vec2::splat(2.0 * control_point_radius);

                let point_in_screen = to_screen.transform_pos(*point);
                let point_rect = Rect::from_center_size(point_in_screen, size);

                let point_id = response.id.with(i);
                let point_click = ui.interact(point_rect, point_id, Sense::click());
                if point_click.clicked() {
                    self.knots_selected[i] = !self.knots_selected[i];
                    clicked = false;
                }

                let min_x = if i > 0 { cp.get(i - 1).unwrap().x } else { 0.0 };
                let max_x = cp.get(i + 1).unwrap_or(&response.rect.max).x;

                let point_response = ui.interact(point_rect, point_id, Sense::drag());

                let delta = point_response.drag_delta();

                if delta != Vec2::ZERO {
                    update = true;
                    *point += point_response.drag_delta();
                    *point = to_screen.from().clamp(*point);
                    point.x = point.x.min(max_x).max(min_x);
                }

                let point_in_screen = to_screen.transform_pos(*point);

                Shape::circle_stroke(
                    point_in_screen,
                    control_point_radius,
                    if self.knots_selected[i] {
                        self.stroke_selected
                    } else {
                        self.stroke_default
                    },
                )
            })
            .collect();

        if clicked {
            // screen position
            let pos = response.interact_pointer_pos().unwrap();

            // data point
            let pos = to_screen.inverse().transform_pos_clamped(pos);

            // insert
            let cp = self
                .knots
                .clone()
                .into_iter()
                .zip(self.knots_selected.clone().into_iter());

            let (head, mut tail): (Vec<_>, Vec<_>) = cp.partition(|(p2, _)| pos.x < p2.x);

            tail.push((pos, false));
            tail.extend(head);

            (self.knots, self.knots_selected) = tail.into_iter().unzip();
            update = true;
        }

        if update {
            self.update();
        }

        if self.knots.len() > 3 {
            let start = self.knots[1].x + 0.00001; // to ensure we have two knots on either side
            let end = self.knots[self.knots.len() - 2][0] - 0.000001;
            // let plot_points = PlotPoints::from_explicit_callback(sample, start..end, 100);

            let interval = end - start;
            let points: u32 = 100;
            let step = interval / (points as f32);

            let mut v = vec![];

            for i in 0..points {
                let t = i as f32 * step + start;
                v.push(
                    to_screen
                        * Pos2 {
                            x: t,
                            y: self.spline.sample(t).unwrap(),
                        },
                )
            }

            painter.add(PathShape::line(v, self.spline_stroke));
        }

        let points_in_screen: Vec<Pos2> = self.knots.iter().map(|p| to_screen * *p).collect();
        painter.add(PathShape::line(points_in_screen, self.line_stroke));
        painter.extend(control_point_shapes);

        response
    }
}
