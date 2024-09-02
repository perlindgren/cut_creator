#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::Stroke;
use egui_plot::{Line, Plot, PlotPoints};

use egui::*;
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Wav Plot",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

use splines::{Interpolation, Key, Spline};

struct MyApp {
    knots: Vec<[f64; 2]>,
    splines: Spline<f64, f64>,
}

impl Default for MyApp {
    fn default() -> Self {
        let knots = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0], [4.0, 1.0], [5.0, 0.0]];
        let splines = Spline::from_iter(
            knots
                .iter()
                .map(|p| Key::new(p[0], p[1], Interpolation::CatmullRom)),
        );

        Self { knots, splines }
    }
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let height = 200.0;
            let width = 300.0;

            ui.horizontal(|ui| {
                let my_plot = Plot::new("My Plot").height(height).width(width);

                my_plot.show(ui, |plot_ui| {
                    if plot_ui.response().clicked() {
                        println!("clicked");

                        if let Some(pos) = plot_ui.response().interact_pointer_pos() {
                            let p2 = plot_ui.pointer_coordinate();
                            println!("pos {:?}, p2 {:?}", pos, p2);

                            if let Some(p2) = p2 {
                                let (head, mut tail): (Vec<[f64; 2]>, Vec<_>) =
                                    self.knots.iter().partition(|k| p2.x < k[0]);

                                println!("head {:?}", head);
                                println!("tail {:?}", tail);

                                tail.push([p2.x, p2.y]);
                                tail.extend(head);
                                self.knots = tail;
                            }

                            println!("{:?}", self.knots);

                            self.splines = Spline::from_iter(
                                self.knots
                                    .iter()
                                    .map(|p| Key::new(p[0], p[1], Interpolation::CatmullRom)),
                            )
                        }
                    }

                    plot_ui.line(Line::new(PlotPoints::from(self.knots.clone())));

                    let control_point_radius = 8.0;

                    // let (response, painter) =
                    //     ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());
                    let response = plot_ui.response();

                    let to_screen = emath::RectTransform::from_to(
                        Rect::from_min_size(Pos2::ZERO, response.rect.size()),
                        response.rect,
                    );

                    let _control_point_shapes: Vec<Shape> = self
                        .knots
                        .iter_mut()
                        .enumerate()
                        .map(|(i, point)| {
                            let size = Vec2::splat(2.0 * control_point_radius);

                            let point_in_screen = to_screen.transform_pos(Pos2 {
                                x: point[0] as f32,
                                y: point[1] as f32,
                            });
                            let _point_rect = Rect::from_center_size(point_in_screen, size);
                            let _point_id = response.id.with(i);
                            // let point_response = ui.interact(point_rect, point_id, Sense::drag());

                            //         // *point += point_response.drag_delta();
                            //         // *point = to_screen.from().clamp(*point);

                            //         // let point_in_screen = to_screen.transform_pos(*point);
                            // let stroke = ui.style().interact(&point_response).fg_stroke;
                            //let stroke = ui.style().interact(response).fg_stroke;
                            let stroke = Stroke::NONE;

                            Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
                        })
                        .collect();

                    // painter.extend(control_point_shapes);

                    // fn f(x: f64) -> f64 {
                    //     (x).sin()
                    // };
                    // let plot_points = PlotPoints::from_explicit_callback(f, (0.0..2.0 * PI), 10);
                    // plot_ui.line(Line::new(plot_points));

                    let splines = self.splines.clone();

                    let sample = move |t| splines.sample(t).unwrap();

                    let start = self.knots[1][0] + 0.00001; // to ensure we have two knots on either side
                    let end = self.knots[self.knots.len() - 2][0] - 0.000001;
                    println!("start {} end {}", start, end);

                    let plot_points = PlotPoints::from_explicit_callback(sample, start..end, 100);
                    plot_ui.line(Line::new(plot_points));
                });
            });
        });
    }
}
