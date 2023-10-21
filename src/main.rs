#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use cut_creator::{
    config::Config,
    cut_panel::Cut,
    wav_panel::{Wav, WavData},
};

// use std::fmt::Display;
// use std::path::Path;
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
    /// we have 10 save slots
    cuts: [Option<(Cut, Wav, WavData)>; 10],
    /// index of selected cut
    cur_cut: usize,
    /// config
    config: Config,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::SidePanel::left("left_id").show(ctx, |ui| {
                ui.vertical(|ui| {
                    for (i, opt_cut) in self.cuts.iter_mut().enumerate() {
                        // let path = opt_cut.map_or("...".to_string(), |(_, p)| p.get_path());
                        let path: &str = if let Some((_c, _w, wd)) = opt_cut {
                            &wd.filename
                        } else {
                            "..."
                        };

                        let button = ui.button(format!("{} {}", i, path,));
                        if button.clicked() {
                            self.cur_cut = i;
                        }
                        if button.double_clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("wav", &["wav"])
                                .set_directory("./audio/")
                                .pick_file()
                            {
                                println!("path {:?}", path);
                                let (w, wd) = Wav::load(path);
                                *opt_cut = Some((Cut::default(), w, wd));
                            }
                        }
                    }
                    ui.separator();

                    ui.checkbox(&mut self.config.knot_line, "knot lines");

                    let mut text = format!("{}", self.config.step_size);
                    ui.horizontal(|ui| {
                        ui.label("Step Size");
                        ui.add(egui::TextEdit::singleline(&mut text));
                    });
                    self.config.step_size = text.parse().unwrap_or(self.config.step_size);

                    if let Some((cut, wav, wav_data)) = &mut self.cuts[self.cur_cut] {
                        wav.ui_content_ctrl(ui, wav_data);

                        cut.ui_content_settings(ui);
                    }
                });
            });
            egui::CentralPanel::default().show(ctx, |_ui| {
                let opt_cut = self.cuts.get_mut(self.cur_cut).unwrap();

                // the waveform
                egui::SidePanel::right("right_id")
                    .frame(egui::Frame::default().inner_margin(egui::Margin::same(5.0)))
                    .show(ctx, |ui| {
                        // main wave panel
                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            if let Some((cut, wav, wav_data)) = opt_cut {
                                wav.ui_content(ui, cut, wav_data, &self.config);
                            }
                        });
                    });

                // the cut panel
                egui::CentralPanel::default()
                    .frame(egui::Frame::default().inner_margin(egui::Margin::same(5.0)))
                    .show(ctx, |ui| {
                        // main cut panel
                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            if let Some((cut, _wav, _wav_data)) = opt_cut {
                                cut.ui_content(ui, &self.config);
                            }
                        });
                    });
            });
        });
    }
}
