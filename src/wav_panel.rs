use crate::cut_panel::Cut;
use egui::epaint::PathShape;
use egui::*;
use std::fs::File;
use std::path::Path;
use wav::{BitDepth, Header};

pub struct Wav {
    stroke_default: Stroke,
    stroke_sample: Stroke,
    _header: Header,
    _stereo: Vec<f32>,
    left: Vec<f32>,
    right: Vec<f32>,
    offset: usize, // in samples, wrapping
    len: usize,    // in samples
}

impl Wav {
    // pub fn get_offset(&self) -> usize {
    //     self.offset
    // }
    // pub fn set_offset(&mut self, offset: usize) {
    //     self.offset = offset;
    // }
    // pub fn get_len(&self) -> usize {
    //     self.len
    // }
    // pub fn set_len(&mut self, len: usize) {
    //     self.len = len;
    // }
    // pub fn get_sample_len(&self) -> usize {
    //     self.left.len()
    // }
}

impl Default for Wav {
    fn default() -> Self {
        let mut inp_file = File::open(Path::new("audio/ahh.wav")).unwrap();
        let (_header, data) = wav::read(&mut inp_file).unwrap();
        println!("header {:?}", _header);

        let _stereo = match data {
            BitDepth::ThirtyTwoFloat(v) => {
                println!("len total{}", v.len());
                v
            }
            _ => {
                vec![]
            }
        };

        let mut v = _stereo.iter();
        let mut left = vec![];
        let mut right = vec![];
        while let Some(l) = v.next() {
            left.push(*l);
            right.push(*v.next().unwrap())
        }
        let len = left.len();
        println!("len samples{}", len);

        Self {
            stroke_default: Stroke::new(1.0, Color32::WHITE),
            stroke_sample: Stroke::new(1.0, Color32::GREEN.linear_multiply(0.25)),
            _header,
            _stereo,
            left,
            right,
            offset: 0,
            len,
        }
    }
}

impl Wav {
    /// control panel
    pub fn ui_content_ctrl(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                self.offset = 0;
            }
            ui.label(format!("offset {}", self.offset));
            // ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
            //     ui.add_sized(
            //         [100.0, 10.0],
            //         egui::Label::new(format!("offset {}", self.get_offset())),
            //     );
            // });
            ui.separator();
            if ui.button("X").clicked() {
                self.len = self.left.len()
            }
            ui.label(format!("len {}", self.len));
        });
    }

    /// main panel
    pub fn ui_content(&mut self, ui: &mut Ui, cut: &Cut) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), ui.available_height()),
            Sense::click_and_drag(),
        );

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let width = response.rect.width();
        let height = response.rect.height();

        // length
        if response.dragged_by(PointerButton::Secondary) {
            let delta = response.drag_delta();
            let delta_scale = ((delta.y / height) * self.len as f32) as i32 as usize;

            self.len = (self.len - delta_scale).max(10_000).min(self.left.len());
            assert!(self.len >= 10_000 && self.len <= self.left.len());
        }

        // offset
        if response.dragged_by(PointerButton::Primary) {
            let delta = response.drag_delta();

            let delta_scale = ((delta.y / height) * self.len as f32) as i32 as usize;

            self.offset = (self.left.len() + self.offset - delta_scale) % self.left.len();
            assert!(self.offset <= self.left.len());
        }

        // compute left/right sample
        let mut left: Vec<Pos2> = vec![];
        let mut right: Vec<Pos2> = vec![];

        let step = self.len as f32 / height;

        for i in 0..height as usize {
            let t =
                (((i as f32) * step) as usize + self.offset + self.left.len()) % self.left.len();
            assert!(t <= self.left.len());

            let l: f32 = self.left[t];
            let r: f32 = self.right[t];
            left.push(
                to_screen
                    * Pos2 {
                        x: l * width * 0.5 + width * 0.25,
                        y: i as f32,
                    },
            );
            right.push(
                to_screen
                    * Pos2 {
                        x: r * width * 0.5 + width * 0.75,
                        y: i as f32,
                    },
            );
        }

        // paint left sample
        painter.add(PathShape::line(left, self.stroke_sample));
        // paint right sample
        painter.add(PathShape::line(right, self.stroke_sample));

        // paint cursor line
        if let Some(cursor) = cut.get_cursor() {
            painter.add(PathShape::line(
                vec![
                    Pos2 {
                        x: response.rect.left(),
                        y: cursor.y,
                    },
                    Pos2 {
                        x: response.rect.right(),
                        y: cursor.y,
                    },
                ],
                self.stroke_default,
            ));
        }

        // paint value
        if let Some(value) = cut.get_value() {
            painter.add(PathShape::line(
                vec![
                    to_screen
                        * Pos2 {
                            x: 0.0,
                            y: value * height,
                        },
                    to_screen
                        * Pos2 {
                            x: width,
                            y: value * height,
                        },
                ],
                self.stroke_default,
            ));
        }

        response
    }
}
