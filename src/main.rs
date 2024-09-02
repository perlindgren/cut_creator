// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::fs::File;
use std::io::prelude::*;

use cut_creator::{
    config::Config,
    cut_panel::{Cut, OptCut},
};

use egui::*;
use log::{debug, info};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    info!("env_logger started");

    let options = eframe::NativeOptions {
        // initial_window_size: Some(egui::vec2(1000.0, 500.0)),
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_min_inner_size([400.0, 200.0]),

        ..Default::default()
    };

    let mut app = Box::<App>::default();
    // load config
    if let Ok(mut file) = File::open("config.json") {
        let mut json = String::new();
        file.read_to_string(&mut json).unwrap();
        debug!("config json {}", json);
        app.config = serde_json::from_str(&json).unwrap();
    }

    eframe::run_native(
        "Cut Creator",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(_cc)))),
    )
}

const NR_EDITORS: usize = 10;
#[derive(Default)]
struct App {
    /// Enable/disable each cut slots
    enabled: [bool; NR_EDITORS],
    /// We have NR_EDITORS save slots
    cuts: [OptCut; NR_EDITORS],
    /// Index of selected cut
    cur_cut: usize,
    /// Top level configuration
    config: Config,
    /// Allowed to close (depending on save state)
    allowed_to_close: bool,
    /// Set if confirmation dialogue already open
    show_confirmation_dialog: bool,
    /// Status string at bottom
    status: String,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }

    /// load file
    fn load_file(&mut self, i: usize) {
        debug!("load_file cut #{}", i);
        self.status = match Cut::load_file() {
            Ok((cut, msg)) => {
                let path = cut.cut_path.clone();
                self.cuts[self.cur_cut] = OptCut(Some(cut));
                self.enabled[i] = true;
                format!("File loaded {}, ({})", path.to_string_lossy(), msg)
            }
            Err(err) => err,
        };
    }

    /// clear all cuts but i
    fn clear_cuts(&mut self, i: usize) {
        debug!("clear_cuts all but {}", i);
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
        debug!("exit");
        // Serialize it to a JSON string.
        let json = serde_json::to_string(&self.config).unwrap();
        debug!("json config {}", json);

        let mut file = File::create("config.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    /// update
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // close?
        if ctx.input(|i| i.viewport().close_requested())
            && !self.allowed_to_close // not allowed to close
            && self.cuts.iter().any(|opt_cut| { // unsaved work?
                if let Some(ref cut) = opt_cut.0 {
                    cut.needs_save()
                } else {
                    false
                }
            })
        {
            // we hav unsaved cuts, so cancel the close event
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_confirmation_dialog = true;
        }

        // close dialog
        if self.show_confirmation_dialog {
            // Show confirmation dialog:
            egui::Window::new("You have unsaved cuts!")
                .anchor(Align2::CENTER_CENTER, [0., 0.])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(25.0);
                        if ui.button("Cancel").clicked() {
                            self.show_confirmation_dialog = false;
                        }
                        ui.add_space(40.0);

                        if ui.button("Quit").clicked() {
                            self.allowed_to_close = true;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
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
                        // each cut has a corresponding button
                        let button = ui.selectable_label(
                            self.enabled[i],
                            format!("#{}: {}", i, opt_cut.get_name()),
                        );

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

                    // Ctrl-O to load file
                    if ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::O)) {
                        opt_load_file = Some(self.cur_cut);
                    }

                    if let Some(i) = opt_load_file {
                        self.load_file(i);
                    }

                    ui.separator();
                    ui.label("Cut Settings");
                    ui.add_space(10.0);

                    if let Some(cut) = &mut self.cuts[self.cur_cut].0 {
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
                        let top_panel_info = match self.cuts[self.cur_cut].0 {
                            Some(ref cut) => {
                                format!(
                                    "#{}: {}, Undo #{:5}, Redo #{}",
                                    self.cur_cut,
                                    cut.get_name(),
                                    cut.get_undo_len(),
                                    cut.get_redo_len()
                                )
                            }
                            _ => "".to_string(),
                        };
                        ui.label(top_panel_info);
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

                    // Very busy tracing, so not enabled
                    // trace!(
                    //     "nr enabled {} height {}, cut_height {}",
                    //     nr_enabled,
                    //     height,
                    //     cut_height
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
                                let opt_cut = &mut self.cuts[i];

                                if *enabled {
                                    egui::Frame::canvas(ui.style())
                                        .outer_margin(egui::Margin::same(3.0))
                                        .inner_margin(egui::Margin::same(0.0))
                                        .show(ui, |ui| {
                                            if let Some(ref mut cut) = opt_cut.0 {
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
                                            if let Some(ref mut cut) = opt_cut.0 {
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
