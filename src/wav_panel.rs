use crate::{config::Config, cut_panel::Cut};
use egui::epaint::PathShape;
use egui::*;
use serde::{Deserialize, Serialize};

use std::{fs::File, path::PathBuf};
use wav::{BitDepth, Header};

pub struct WavData {
    _header: Header,
    _stereo: Vec<f32>,
    left: Vec<f32>,
    right: Vec<f32>,
    /// the max length
    len: usize,
    /// filename
    pub filename: String,
}

#[derive(Serialize, Deserialize)]
pub struct Wav {
    path: PathBuf,
    offset: usize, // in samples, wrapping
    len: usize,    // in samples
}

impl Wav {
    /// load
    pub fn load(path: PathBuf) -> (Self, WavData) {
        let mut inp_file = File::open(&path).unwrap();
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
        let filename = path.file_stem().unwrap().to_str().unwrap().to_owned();

        (
            Wav {
                path,
                offset: 0,
                len,
            },
            WavData {
                _header,
                _stereo,
                left,
                right,
                len,
                filename,
            },
        )
    }

    /// get path
    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    /// control panel
    pub fn ui_content_ctrl(&mut self, ui: &mut Ui, wav_data: &WavData, i: usize) {
        ui.label(format!("#{}: {}", i, wav_data.filename));
        // ui.label(format!("{}", self.path.display()));

        ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                self.offset = 0;
            }
            ui.label(format!("offset {}", self.offset));
        });

        ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                self.len = wav_data.len
            }
            ui.label(format!("len {}", self.len));
        });
    }

    /// main panel
    pub fn ui_content(
        &mut self,
        ui: &mut Ui,
        cut: &Cut,
        wav_data: &WavData,
        config: &Config,
        height: f32,
    ) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), height),
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

            self.len = (self.len - delta_scale).max(10_000).min(wav_data.len);
            assert!(self.len >= 10_000 && self.len <= wav_data.len);
        }

        // offset
        if response.dragged_by(PointerButton::Primary) {
            let delta = response.drag_delta();

            let delta_scale = ((delta.y / height) * self.len as f32) as i32 as usize;

            self.offset = (wav_data.len + self.offset - delta_scale) % wav_data.len;
            assert!(self.offset <= wav_data.len);
        }

        // compute left/right sample
        let mut left: Vec<Pos2> = vec![];
        let mut right: Vec<Pos2> = vec![];

        let step = self.len as f32 / height;

        for i in 0..height as usize {
            let t = (((i as f32) * step) as usize + self.offset + wav_data.len) % wav_data.len;
            assert!(t <= wav_data.len);

            let l: f32 = wav_data.left[t];
            let r: f32 = wav_data.right[t];
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
        painter.add(PathShape::line(left, config.stroke_sample));
        // paint right sample
        painter.add(PathShape::line(right, config.stroke_sample));

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
                config.stroke_line,
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
                config.stroke_spline,
            ));
        }

        response
    }
}
