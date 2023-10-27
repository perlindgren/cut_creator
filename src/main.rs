#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::fs::File;
use std::io::prelude::*;

use cut_creator::{config::Config, cut_panel::Cut};

use egui::*;
use log::trace;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 500.0)),
        min_window_size: Some(Vec2::new(400.0, 200.0)),
        ..Default::default()
    };

    let mut app = Box::<App>::default();
    // load config
    if let Ok(mut file) = File::open("config.json") {
        let mut json = String::new();
        file.read_to_string(&mut json).unwrap();
        trace!("config json {}", json);
        app.config = serde_json::from_str(&json).unwrap();
    }

    eframe::run_native("Cut Creator", options, Box::new(|_cc| app))
}

#[derive(Default)]
struct App {
    ///
    enabled: [bool; 10],
    /// we have 10 save slots
    cuts: [Option<Cut>; 10],
    /// index of selected cut
    cur_cut: usize,
    /// config
    config: Config,
    ///
    allowed_to_close: bool,
    ///
    show_confirmation_dialog: bool,
    /// status bottom
    status: String,
}

impl App {
    /// load file
    fn load_file(&mut self, i: usize) {
        self.status = match Cut::load_file() {
            Ok(cut) => {
                let path = cut.cut_path.clone();
                self.cuts[self.cur_cut] = Some(cut);
                self.enabled[i] = true;
                format!("File loaded {}", path.to_string_lossy())
            }
            Err(err) => err,
        };
    }

    /// clear all cuts but i
    fn clear_cuts(&mut self, i: usize) {
        for (index, enable) in self.enabled.iter_mut().enumerate() {
            if index != i {
                *enable = false
            };
        }
    }
}

impl eframe::App for App {
    /// on exit
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        trace!("exit");
        // Serialize it to a JSON string.
        let json = serde_json::to_string(&self.config).unwrap();
        trace!("json config {}", json);

        let mut file = File::create("config.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    /// on close event
    fn on_close_event(&mut self) -> bool {
        trace!("close");
        if self.cuts.iter().any(|opt_cut| {
            if let Some(cut) = opt_cut {
                cut.needs_save
            } else {
                false
            }
        }) {
            self.show_confirmation_dialog = true;
            self.allowed_to_close
        } else {
            true
        }
    }

    /// update
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // close dialog
        if self.show_confirmation_dialog {
            // Show confirmation dialog:
            egui::Window::new("You have unsaved cuts")
                .anchor(Align2::CENTER_CENTER, [0., 0.])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_confirmation_dialog = false;
                        }

                        if ui.button("Quit").clicked() {
                            self.allowed_to_close = true;
                            frame.close();
                        }
                    });
                });
        }
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
                        self.clear_cuts(10);
                    }

                    // enabling/disabling cuts
                    //
                    // on first click load cut
                    // consecutive click to select cut as active
                    // shift click to multi select cuts
                    // double click allows to load new sample

                    let mut opt_load_file: Option<usize> = None;

                    for (i, opt_cut) in self.cuts.iter_mut().enumerate() {
                        let path = if let Some(cut) = opt_cut {
                            format!("{}{}", cut.name(), if cut.needs_save { "*" } else { "" })
                        } else {
                            "...".to_string()
                        };

                        // each cut has a corresponding button
                        let button =
                            ui.selectable_label(self.enabled[i], format!("#{}: {}", i, path));

                        // check hover
                        if button.interact(Sense::hover()).hovered() {
                            self.cur_cut = i;
                        }

                        if button.clicked() {
                            self.enabled[i] ^= true;
                        }

                        if button.double_clicked() {
                            opt_load_file = Some(i)
                        }

                        if self.cur_cut == i {
                            button.highlight();
                        };
                    }

                    ui.input(|is| {
                        if is.key_pressed(Key::Enter) {
                            self.enabled[self.cur_cut] ^= true;
                        }
                    });

                    if ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::O)) {
                        opt_load_file = Some(self.cur_cut);
                    }

                    if let Some(i) = opt_load_file {
                        self.load_file(i);
                    }

                    ui.separator();
                    ui.label("Cut Settings");
                    ui.add_space(10.0);

                    if let Some(cut) = &mut self.cuts[self.cur_cut] {
                        cut.wav.ui_content_ctrl(ui, &cut.wav_data, self.cur_cut);

                        cut.ui_content_settings(ui, &mut self.status);
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
                        ui.label(&self.status);
                    });

                // populate center panel only if some cuts are enabled
                if nr_enabled > 0 {
                    let height = ui.available_height();
                    let _width = ui.available_width();
                    let cut_height = (height - 20.0 - nr_enabled as f32 * 10.0) / nr_enabled as f32;
                    trace!(
                        "nr enabled {} height {}, cut_height {}",
                        nr_enabled,
                        height,
                        cut_height
                    );

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
                                let opt_cut = &mut self.cuts[i];

                                if *enabled {
                                    egui::Frame::canvas(ui.style())
                                        .outer_margin(egui::Margin::same(3.0))
                                        .inner_margin(egui::Margin::same(0.0))
                                        .show(ui, |ui| {
                                            if let Some(cut) = opt_cut {
                                                cut.wav.ui_content(
                                                    ui,
                                                    cut.get_cursor(),
                                                    cut.get_value(),
                                                    &cut.wav_data,
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
                                            if let Some(cut) = opt_cut {
                                                cut.ui_content(ui, &self.config, cut_height);
                                            }
                                        });
                                }
                            }
                        });
                }
            });
        });
    }
}
