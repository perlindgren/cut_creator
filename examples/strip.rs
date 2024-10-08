#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui_extras::{Size, StripBuilder};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App Per",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

#[derive(Default)]
struct MyApp {}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::remainder().at_least(40.0)) // top cell
                .size(Size::exact(40.0)) // bottom cell
                .vertical(|mut strip| {
                    // Add the top 'cell'
                    strip.cell(|ui| {
                        ui.label("Fixed");
                    });

                    // We add a nested strip in the bottom cell:
                    strip.strip(|builder| {
                        builder
                            //.sizes(Size::remainder().at_most(40.0).at_least(40.0), 2)
                            .sizes(Size::relative(0.5), 2)
                            .horizontal(|mut strip| {
                                strip.cell(|ui| {
                                    ui.label("Left");
                                });
                                // strip.cell(|ui| {
                                //     ui.label("Middle");
                                // });
                                strip.cell(|ui| {
                                    ui.label("Right");
                                });
                            });
                    });
                });
        });
    }
}
