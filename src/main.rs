#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use cut_creator::{cut_panel::Cut, cut_settings::CutSettings, wav_panel::Wav};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 500.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Cut Creator",
        options,
        Box::new(|_cc| Box::<App>::default()),
    )
}

#[derive(Default)]
struct App {
    cut_settings: CutSettings,
    cut: Cut,
    wav: Wav,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|_ui| {
                // the waveform
                egui::SidePanel::right("Right").show(ctx, |ui| {
                    ui.heading("Audio");
                    ui.vertical_centered(|ui| {
                        egui::Frame::canvas(ui.style()).show(ui, |_ui| {
                            //
                        });
                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            ui.set_min_width(100.0);
                            self.wav.ui_content(ui, &self.cut);
                        });
                    });
                });

                // the cut panel
                egui::CentralPanel::default().show(ctx, |ui| {
                    //ui.heading("Cut");
                    ui.vertical_centered(|ui| {
                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            self.cut_settings.ui_content(ui);
                        });
                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            self.cut.ui_content(ui, self.cut_settings);
                        })
                    });
                });
            });
        });
    }
}
