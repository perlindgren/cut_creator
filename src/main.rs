#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use cut_creator::{
    config::Config,
    cut_panel::Cut,
    wav_panel::{Wav, WavData},
};

use egui::*;

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
    ///
    enabled: [bool; 10],
    /// we have 10 save slots
    cuts: [Option<(Cut, Wav, WavData)>; 10],
    /// index of selected cut
    cur_cut: usize,
    /// config
    config: Config,
}

// helper
fn clear_cuts(enabled: &mut [bool; 10], i: usize) {
    for (index, enable) in enabled.iter_mut().enumerate() {
        if index != i {
            *enable = false
        };
    }
}

fn load(opt_cut: &mut Option<(Cut, Wav, WavData)>) {
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_ui| {
            // left side panel
            egui::SidePanel::left("left_id").show(ctx, |ui| {
                ui.vertical(|ui| {
                    // keyboard events

                    if !ctx.wants_keyboard_input() {
                        ui.input(|i| {
                            if i.key_pressed(Key::ArrowDown) {
                                self.cur_cut = (self.cur_cut + 1) % 10;
                            }

                            // num key shortcut
                            if i.key_pressed(Key::ArrowUp) {
                                self.cur_cut = (10 + self.cur_cut - 1) % 10;
                            }

                            use Key::*;
                            let dig = [Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9];

                            dig.iter().enumerate().for_each(|(num, key)| {
                                if i.key_pressed(*key) {
                                    self.cur_cut = num;
                                    self.enabled[num] ^= true;
                                    // if !i.modifiers.contains(Modifiers::SHIFT) {
                                    //     println!("clear cuts");
                                    //     clear_cuts(&mut self.enabled, num)
                                    // } else {
                                    //     println!("SHIFT, do not clear cuts");
                                    // }
                                }
                            })
                        });
                    }

                    ui.label("Settings");
                    ui.add_space(10.0);
                    ui.checkbox(&mut self.config.knot_line, "knot lines");

                    let mut text = format!("{}", self.config.step_size);
                    ui.horizontal(|ui| {
                        ui.label("Step Size");
                        if ui.add(egui::TextEdit::singleline(&mut text)).changed() {
                            self.config.step_size = text.parse().unwrap_or(self.config.step_size);
                        }
                    });

                    ui.separator();

                    ui.label("Selected Cuts");
                    ui.add_space(10.0);

                    // clear all selected
                    if ui.button("Clear all selected cuts").clicked() {
                        clear_cuts(&mut self.enabled, 10);
                    }

                    // enabling/disabling cuts
                    //
                    // on first click load cut
                    // consecutive click to select cut as active
                    // shift click to multi select cuts
                    // double click allows to load new sample
                    for (i, opt_cut) in self.cuts.iter_mut().enumerate() {
                        // let path = opt_cut.map_or("...".to_string(), |(_, p)| p.get_path());
                        let path: &str = if let Some((_c, _w, wd)) = opt_cut {
                            &wd.filename
                        } else {
                            "..."
                        };

                        // each cut has a corresponding button
                        let button =
                            ui.selectable_label(self.enabled[i], format!("#{}: {}", i, path,));

                        // check hover
                        if button.interact(Sense::hover()).hovered() {
                            self.cur_cut = i;
                        }

                        if button.clicked() {
                            self.enabled[i] ^= true;
                        }

                        ui.input(|is| {
                            if is.key_pressed(Key::Enter) && i == self.cur_cut {
                                self.enabled[i] ^= true;
                            }
                        });

                        // load cut
                        if button.double_clicked() || (self.enabled[i] && opt_cut.is_none()) {
                            load(opt_cut);
                        }

                        if self.cur_cut == i {
                            button.highlight();
                        };
                    }

                    ui.separator();
                    ui.label("Cut Settings");
                    ui.add_space(10.0);

                    if let Some((cut, wav, wav_data)) = &mut self.cuts[self.cur_cut] {
                        wav.ui_content_ctrl(ui, wav_data, self.cur_cut);

                        cut.ui_content_settings(ui);
                    }
                });
            });

            // center panel
            egui::CentralPanel::default().show(ctx, |ui| {
                // count number of enabled cuts
                let nr_enabled = self
                    .enabled
                    .iter()
                    .fold(0, |acc, b| acc + if *b { 1 } else { 0 });

                // dummy top
                egui::TopBottomPanel::top("top")
                    .frame(
                        Frame::default()
                            .outer_margin(egui::Margin::same(0.0))
                            .inner_margin(egui::Margin::same(0.0)),
                    )
                    .show(ctx, |ui| {
                        ui.label("dummy top panel, let's see what to do with that");
                    });

                // dummy bottom
                egui::TopBottomPanel::bottom("bottom")
                    .frame(
                        Frame::default()
                            .outer_margin(egui::Margin::same(0.0))
                            .inner_margin(egui::Margin::same(0.0)),
                    )
                    .show(ctx, |ui| {
                        ui.label("dummy bottom panel, let's see what to do with that");
                    });

                // populate center panel only if some cuts are enabled
                if nr_enabled > 0 {
                    let height = ui.available_height();
                    let _width = ui.available_width();
                    let cut_height = (height - 20.0 - nr_enabled as f32 * 10.0) / nr_enabled as f32;
                    // println!(
                    //     "nr enabled {} height {}, cut_height {}",
                    //     nr_enabled, height, cut_height
                    // );

                    // right side panel with wav
                    egui::SidePanel::right("right")
                        .frame(
                            Frame::default()
                                .outer_margin(egui::Margin::same(0.0))
                                .inner_margin(egui::Margin::same(0.0)),
                        )
                        .show(ctx, |ui| {
                            // right wave panel
                            for (i, enabled) in self.enabled.iter().enumerate() {
                                let opt_cut = self.cuts.get_mut(i).unwrap();
                                if *enabled {
                                    egui::Frame::canvas(ui.style())
                                        .outer_margin(egui::Margin::same(3.0))
                                        .inner_margin(egui::Margin::same(0.0))
                                        .show(ui, |ui| {
                                            if let Some((cut, wav, wav_data)) = opt_cut {
                                                wav.ui_content(
                                                    ui,
                                                    cut,
                                                    wav_data,
                                                    &self.config,
                                                    cut_height,
                                                );
                                            }
                                        });
                                }
                            }
                        });

                    // the cut panel
                    egui::CentralPanel::default()
                        .frame(
                            Frame::default()
                                .outer_margin(egui::Margin::same(0.0))
                                .inner_margin(egui::Margin::same(0.0)),
                        )
                        .show(ctx, |ui| {
                            // main cut panel
                            for (i, enabled) in self.enabled.iter().enumerate() {
                                let opt_cut = self.cuts.get_mut(i).unwrap();
                                if *enabled {
                                    egui::Frame::canvas(ui.style())
                                        .outer_margin(egui::Margin::same(3.0))
                                        .inner_margin(egui::Margin::same(0.0))
                                        .show(ui, |ui| {
                                            if let Some((cut, _wav, _wav_data)) = opt_cut {
                                                cut.ui_content(ui, &self.config, cut_height);
                                            }
                                        });
                                }
                            }
                        });
                }
            });
        });

        // for (i, enabled) in self.enabled.iter().enumerate() {
        //     if *enabled {
        //         let mut ui = ui.child_ui(
        //             Rect {
        //                 min: Pos2::ZERO,
        //                 max: Pos2 {
        //                     x: ui.available_width(),
        //                     y: cut_height,
        //                 },
        //             },

        //         let opt_cut = self.cuts.get_mut(i).unwrap();
        //         ui.set_height(cut_height);
        //         ui.allocate_ui(egui::Vec2::new(200.0, cut_height), |ui| {
        //             egui::SidePanel::right(egui::Id::new(i))
        //                 .frame(
        //                     egui::Frame::default()
        //                         .inner_margin(egui::Margin::same(5.0)),
        //                 )
        //                 .show(ctx, |ui| {
        //                     // right wave panel
        //                     egui::Frame::canvas(ui.style()).show(ui, |ui| {
        //                         if let Some((cut, wav, wav_data)) = opt_cut {
        //                             wav.ui_content(ui, cut, wav_data, &self.config);
        //                         }
        //                     });
        //                 });

        //             // the cut panel
        //             egui::CentralPanel::default()
        //                 .frame(
        //                     egui::Frame::default()
        //                         .inner_margin(egui::Margin::same(5.0)),
        //                 )
        //                 //.show_inside(ui, |ui| {
        //                 .show(ctx, |ui| {
        //                     // main cut panel
        //                     egui::Frame::canvas(ui.style()).show(ui, |ui| {
        //                         if let Some((cut, _wav, _wav_data)) = opt_cut {
        //                             cut.ui_content(ui, &self.config, height);
        //                         }
        //                     });
        //                 });
        //             // });
        //         });
        //     }
        // }
        //}
        //});
        //});
    }
}
