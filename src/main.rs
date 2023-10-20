#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use cut_creator::{cut_panel::Cut, wav_panel::Wav};

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
    cut: Cut,
    wav: Wav,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::TopBottomPanel::bottom("bottom_id")
                .frame(egui::Frame::default().inner_margin(egui::Margin::same(8.0)))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        // settings
                        self.cut.ui_content_settings(ui);

                        ui.separator();

                        for i in 0..12 {
                            if ui.button(format!("{}", i)).changed() {
                                println!("i {}", i);
                            }
                        }
                        ui.separator();

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                            // control for wav
                            self.wav.ui_content_ctrl(ui);
                        });
                    });
                });
            egui::CentralPanel::default().show(ctx, |_ui| {
                // the waveform
                egui::SidePanel::right("right_id")
                    .frame(egui::Frame::default().inner_margin(egui::Margin::same(5.0)))
                    .show(ctx, |ui| {
                        // main wave panel
                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            self.wav.ui_content(ui, &self.cut);
                        });
                    });

                // the cut panel
                egui::CentralPanel::default()
                    .frame(egui::Frame::default().inner_margin(egui::Margin::same(5.0)))
                    .show(ctx, |ui| {
                        // main cut panel
                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            self.cut.ui_content(ui);
                        });
                    });
            });
        });
    }
}
