#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, egui::ColorImage};
use egui::plot::{Legend, Line, Plot, PlotPoints};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(350.0, 400.0)),
        ..Default::default()
    };
    eframe::run_native("Wav Plot", options, Box::new(|_cc| Box::<MyApp>::default()))
}

#[derive(Default)]
struct MyApp {
    screenshot: Option<ColorImage>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut plot_rect = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            // these are just some dummy variables for the example,
            // such that the plot is not at position (0,0)
            let height = 200.0;
            let border_x = 11.0;
            let border_y = 18.0;
            let width = 3000.0;

            ui.heading("Wav Plot");

            // add some whitespace in y direction
            ui.add_space(border_y);

            ui.horizontal(|ui| {
                if ui.button("Save Plot").clicked() {
                    frame.request_screenshot();
                }
                if ui.button("Load Plot").clicked() {
                    frame.request_screenshot();
                }
            });

            // add some whitespace in y direction
            ui.add_space(border_y);

            ui.horizontal(|ui| {
                // add some whitespace in x direction
                ui.add_space(border_x);

                let my_plot = Plot::new("My Plot")
                    .height(height)
                    .width(width)
                    .legend(Legend::default());

                // let's create a dummy line in the plot
                let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
                let inner = my_plot.show(ui, |plot_ui| {
                    plot_ui.line(Line::new(PlotPoints::from(graph)).name("curve"));
                });
                // Remember the position of the plot
                plot_rect = Some(inner.response.rect);
            });

            // add some whitespace in y direction
            ui.add_space(border_y);
        });

        if let (Some(screenshot), Some(plot_location)) = (self.screenshot.take(), plot_rect) {
            if let Some(mut path) = rfd::FileDialog::new().save_file() {
                path.set_extension("png");

                // for a full size application, we should put this in a different thread,
                // so that the GUI doesn't lag during saving

                let pixels_per_point = frame.info().native_pixels_per_point;
                let plot = screenshot.region(&plot_location, pixels_per_point);
                // save the plot to png
                image::save_buffer(
                    &path,
                    plot.as_raw(),
                    plot.width() as u32,
                    plot.height() as u32,
                    image::ColorType::Rgba8,
                )
                .unwrap();
            }
        }
    }

    fn post_rendering(&mut self, _screen_size_px: [u32; 2], frame: &eframe::Frame) {
        // this is inspired by the Egui screenshot example
        if let Some(screenshot) = frame.screenshot() {
            self.screenshot = Some(screenshot);
        }
    }
}

use egui::epaint::{CubicBezierShape, PathShape, QuadraticBezierShape};
use egui::*;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct PaintBezier {
    /// Bézier curve degree, it can be 3, 4.
    degree: usize,

    /// The control points. The [`Self::degree`] first of them are used.
    control_points: [Pos2; 4],

    /// Stroke for Bézier curve.
    stroke: Stroke,

    /// Fill for Bézier curve.
    fill: Color32,

    /// Stroke for auxiliary lines.
    aux_stroke: Stroke,

    bounding_box_stroke: Stroke,
}

impl Default for PaintBezier {
    fn default() -> Self {
        Self {
            degree: 4,
            control_points: [
                pos2(50.0, 50.0),
                pos2(60.0, 250.0),
                pos2(200.0, 200.0),
                pos2(250.0, 50.0),
            ],
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            fill: Color32::from_rgb(50, 100, 150).linear_multiply(0.25),
            aux_stroke: Stroke::new(1.0, Color32::RED.linear_multiply(0.25)),
            bounding_box_stroke: Stroke::new(0.0, Color32::LIGHT_GREEN.linear_multiply(0.25)),
        }
    }
}

impl PaintBezier {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Colors", |ui| {
            ui.horizontal(|ui| {
                ui.label("Fill color:");
                ui.color_edit_button_srgba(&mut self.fill);
            });
            egui::stroke_ui(ui, &mut self.stroke, "Curve Stroke");
            egui::stroke_ui(ui, &mut self.aux_stroke, "Auxiliary Stroke");
            egui::stroke_ui(ui, &mut self.bounding_box_stroke, "Bounding Box Stroke");
        });

        ui.collapsing("Global tessellation options", |ui| {
            let mut tessellation_options = ui.ctx().tessellation_options(|to| *to);
            tessellation_options.ui(ui);
            ui.ctx()
                .tessellation_options_mut(|to| *to = tessellation_options);
        });

        ui.radio_value(&mut self.degree, 3, "Quadratic Bézier");
        ui.radio_value(&mut self.degree, 4, "Cubic Bézier");
        ui.label("Move the points by dragging them.");
        ui.small("Only convex curves can be accurately filled.");
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let control_point_radius = 8.0;

        let control_point_shapes: Vec<Shape> = self
            .control_points
            .iter_mut()
            .enumerate()
            .take(self.degree)
            .map(|(i, point)| {
                let size = Vec2::splat(2.0 * control_point_radius);

                let point_in_screen = to_screen.transform_pos(*point);
                let point_rect = Rect::from_center_size(point_in_screen, size);
                let point_id = response.id.with(i);
                let point_response = ui.interact(point_rect, point_id, Sense::drag());

                *point += point_response.drag_delta();
                *point = to_screen.from().clamp(*point);

                let point_in_screen = to_screen.transform_pos(*point);
                let stroke = ui.style().interact(&point_response).fg_stroke;

                Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
            })
            .collect();

        let points_in_screen: Vec<Pos2> = self
            .control_points
            .iter()
            .take(self.degree)
            .map(|p| to_screen * *p)
            .collect();

        match self.degree {
            3 => {
                let points = points_in_screen.clone().try_into().unwrap();
                let shape =
                    QuadraticBezierShape::from_points_stroke(points, true, self.fill, self.stroke);
                painter.add(epaint::RectShape::stroke(
                    shape.visual_bounding_rect(),
                    0.0,
                    self.bounding_box_stroke,
                ));
                painter.add(shape);
            }
            4 => {
                let points = points_in_screen.clone().try_into().unwrap();
                let shape =
                    CubicBezierShape::from_points_stroke(points, true, self.fill, self.stroke);
                painter.add(epaint::RectShape::stroke(
                    shape.visual_bounding_rect(),
                    0.0,
                    self.bounding_box_stroke,
                ));
                painter.add(shape);
            }
            _ => {
                unreachable!();
            }
        };

        painter.add(PathShape::line(points_in_screen, self.aux_stroke));
        painter.extend(control_point_shapes);

        response
    }
}
