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

struct MyApp {
    knots: Vec<[f64; 2]>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            knots: vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0], [2.5, 5.0]],
        }
    }
}

use std::ops::Bound::*;
use std::ops::RangeBounds;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let height = 200.0;
            let width = 300.0;

            ui.horizontal(|ui| {
                let my_plot = Plot::new("My Plot").height(height).width(width);

                // let's create a dummy line in the plot
                let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0], [2.5, 5.0]];
                my_plot.show(ui, |plot_ui| {
                    if plot_ui.response().clicked() {
                        println!("clicked");
                    }

                    if let Some(pos) = plot_ui.response().interact_pointer_pos() {
                        let p2 = plot_ui.pointer_coordinate();
                        println!("pos {:?}, p2 {:?}", pos, p2);
                        if let Some(p2) = p2 {
                            self.knots.push([p2.x, p2.y]);
                        }
                    }
                    plot_ui.line(Line::new(PlotPoints::from(self.knots.clone())));
                    fn f(x: f64) -> f64 {
                        (x).sin()
                    };
                    let plot_point = PlotPoints::from_explicit_callback(f, (0.0..2.0 * PI), 10);

                    plot_ui.line(Line::new(plot_point));
                });
            });
        });
    }
}
