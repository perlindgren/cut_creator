use egui::*;

use crate::wav_panel::Wav;

impl Wav {
    pub fn ui_content_ctrl(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                self.set_offset(0);
            }
            ui.label(format!("offset {}", self.get_offset()));

            if ui.button("X").clicked() {
                self.set_len(self.get_sample_len());
            }
            ui.label(format!("len {}", self.get_len()));
        });
    }
}
