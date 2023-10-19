use crate::cut_panel::Cut;
use egui::*;

// settings
#[derive(Copy, Clone, Debug)]
pub struct CutSettings {
    /// looping behavior
    looping: bool,

    /// warping behavior
    warping: bool,
}

impl CutSettings {
    pub fn is_looped(&self) -> bool {
        self.looping
    }

    pub fn is_warped(&self) -> bool {
        self.warping
    }
}

impl Default for CutSettings {
    fn default() -> Self {
        CutSettings {
            looping: false,
            warping: true,
        }
    }
}

impl CutSettings {
    pub fn ui_content(&mut self, ui: &mut Ui, cut: &mut Cut) {
        ui.horizontal(|ui| {
            if ui.checkbox(&mut self.looping, "looping").clicked() {
                println!("looping {}", self.looping);
                cut.update(self);
            }

            if ui.checkbox(&mut self.warping, "warping").clicked() {
                println!("warping {}", self.warping)
            }
        });
    }
}
