#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use cut_creator::{cut_panel::Cut, wav_panel::Wav};
use std::fmt::Display;
use std::path::PathBuf;
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
    cuts: [Option<(Cut, Wav)>; 10],
    /// index of selected cut
    cur_cut: usize,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::SidePanel::left("left_id").show(ctx, |ui| {
                ui.vertical(|ui| {
                    for (i, opt_cut) in self.cuts.iter_mut().enumerate() {
                        // let path = opt_cut.map_or("...".to_string(), |(_, p)| p.get_path());
                        let path = if let Some((_c, w)) = opt_cut {
                            w.get_path()
                        } else {
                            "...".to_string()
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
                                let s: &str = &format!("{:?}", path);
                                println!("path {:?}", s);
                                *opt_cut = Some((Cut::default(), Wav::load(s)));
                            }
                        }
                    }
                    ui.separator();
                    // egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    if let Some((cut, wav)) = &mut self.cuts[self.cur_cut] {
                        // ui.horizontal(|ui| {
                        wav.ui_content_ctrl(ui);
                        // });
                        // ui.horizontal(|ui| {
                        cut.ui_content_settings(ui);
                        // });
                    }
                    // });
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
                            if let Some((cut, wav)) = opt_cut {
                                wav.ui_content(ui, &cut);
                            }
                        });
                    });

                // the cut panel
                egui::CentralPanel::default()
                    .frame(egui::Frame::default().inner_margin(egui::Margin::same(5.0)))
                    .show(ctx, |ui| {
                        // main cut panel
                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            if let Some((cut, wav)) = opt_cut {
                                cut.ui_content(ui);
                            }
                        });
                    });
            });
        });
    }
}
