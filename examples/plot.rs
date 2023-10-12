#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::f64::consts::PI;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 400.0)),
        ..Default::default()
    };
    eframe::run_native("Wav Plot", options, Box::new(|_cc| Box::<MyApp>::default()))
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
