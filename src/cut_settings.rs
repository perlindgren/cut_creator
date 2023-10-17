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
    pub fn ui_content(&mut self, ui: &mut Ui) {
        // let (response, painter) = ui.allocate_painter(
        //     Vec2::new(ui.available_width(), ui.available_height()),
        //     Sense::click_and_drag(),
        // );

        ui.horizontal(|ui| {
            if ui.checkbox(&mut self.looping, "looping").clicked() {
                println!("looping {}", self.looping)
            }

            if ui.checkbox(&mut self.warping, "warping").clicked() {
                println!("warping {}", self.warping)
            }
        });
    }
}
