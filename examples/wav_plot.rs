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
struct MyApp {}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut plot_rect = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            // these are just some dummy variables for the example,
            // such that the plot is not at position (0,0)
            let height = 200.0;
            let border_x = 11.0;
            let border_y = 18.0;
            let width = 300.0;

            ui.heading("Wav Plot");

            // add some whitespace in y direction
            ui.add_space(border_y);

            ui.horizontal(|ui| {
                if ui.button("Save Plot").clicked() {}
                if ui.button("Load Wav").clicked() {}
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
    }
}
