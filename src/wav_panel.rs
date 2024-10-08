use crate::config::Config;
use egui::epaint::PathShape;
use egui::*;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;
// use wav::{BitDepth, Header};
use hound::WavSpec;
use log::{debug, trace};

#[derive(Default, Debug)]
pub struct WavData {
    wav_spec: Option<WavSpec>,
    // _stereo: Vec<f32>,
    left: Vec<f32>,
    right: Vec<f32>,
    /// the max length
    pub len: usize,
    /// filename
    pub filename: String,
}

impl WavData {
    /// load
    pub fn load_wav_data(path: PathBuf) -> Result<Self, String> {
        let filename = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let mut reader = hound::WavReader::open(path)
            .map_err(|_| "Unsupported format (use stereo WAV)".to_string())?;
        let wav_spec = reader.spec();
        trace!("wav_spec {:?}", wav_spec);
        if wav_spec.channels != 2 {
            Err("Only stereo files supported")?;
        }

        let (left, right, _b) =
            reader
                .samples::<f32>()
                .fold((vec![], vec![], false), |(mut l, mut r, b), s| {
                    let s = s.unwrap();
                    if b {
                        l.push(s);
                    } else {
                        r.push(s);
                    }
                    (l, r, !b)
                });

        let len = reader.duration() as usize;
        trace!("duration: len samples {}", len);

        Ok(WavData {
            wav_spec: Some(wav_spec),
            // _stereo,
            left,
            right,
            len,
            filename,
        })
    }

    pub fn get_wav_spec(&self) -> Option<WavSpec> {
        self.wav_spec
    }

    pub fn get_sample(&self, index: usize) -> (f32, f32) {
        (
            *self.left.get(index).unwrap_or(&0.0),
            *self.right.get(index).unwrap_or(&0.0),
        )
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct CheckPointData {
    offset: usize, // in samples, wrapping
    len: usize,    // in samples
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Wav {
    data: CheckPointData,

    #[serde(skip)]
    undo: Vec<CheckPointData>,
    redo: Vec<CheckPointData>,
}

impl Wav {
    /// get_sample at v, where p = 0..data.len
    pub fn get_sample(&self, p: usize, wav_data: &WavData) -> (f32, f32) {
        let t = p + self.data.offset;
        wav_data.get_sample(t)
    }

    /// Get offset
    pub fn get_data_offset(&self) -> usize {
        self.data.offset
    }

    /// set len
    pub fn set_data_len(&mut self, len: usize) {
        self.data.len = len
    }

    /// Get data len
    pub fn get_data_len(&self) -> usize {
        self.data.len
    }

    /// Get undo len
    pub fn get_undo_len(&self) -> usize {
        self.undo.len()
    }

    /// Get redo len
    pub fn get_redo_len(&self) -> usize {
        self.redo.len()
    }

    /// Needs save
    pub fn needs_save(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Clear undo redo buffers
    pub fn clear_undo_redo(&mut self) {
        self.undo = vec![];
        self.redo = vec![];
    }

    /// Control panel for the Wav
    pub fn ui_content_ctrl(&mut self, ui: &mut Ui, wav_data: &WavData, i: usize) {
        ui.label(format!("#{}: {}", i, wav_data.filename));
        // ui.label(format!("{}", self.path.display()));

        ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                self.data.offset = 0;
            }
            ui.label(format!("offset {}", self.data.offset));
        });

        ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                self.data.len = wav_data.len
            }
            ui.label(format!("len {}", self.data.len));
        });
    }

    /// main panel
    pub fn ui_content(
        &mut self,
        ui: &mut Ui,
        cut_cursor: Option<Pos2>,
        cut_value: Option<f32>,
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
        if response.drag_started_by(PointerButton::Secondary) {
            debug!("length drag_started_by");
            self.undo.push(self.data.clone());
        }

        // length
        if response.dragged_by(PointerButton::Secondary) {
            trace!("length dragged_by");
            let delta = response.drag_delta();
            let delta_scale = ((delta.y / height) * self.data.len as f32) as i32 as usize;

            self.data.len = (self.data.len - delta_scale).max(10_000).min(wav_data.len);
            assert!(self.data.len >= 10_000 && self.data.len <= wav_data.len);
        }

        // offset
        if response.drag_started_by(PointerButton::Primary) {
            debug!("offset drag_started_by");
            self.undo.push(self.data.clone());
        }
        // offset
        if response.dragged_by(PointerButton::Primary) {
            trace!("offset dragged_by");
            let delta = response.drag_delta();

            let delta_scale = ((delta.y / height) * self.data.len as f32) as i32 as usize;

            self.data.offset = (wav_data.len + self.data.offset - delta_scale) % wav_data.len;
            assert!(self.data.offset <= wav_data.len);
        }

        if response.hovered() {
            // undo checkpoint
            if ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::Z)) {
                debug!("Ctrl-Z");
                if let Some(check_point) = self.undo.pop() {
                    self.redo.push(self.data.clone());
                    self.data = check_point;
                }
            }

            // redo checkpoint
            if ui.input_mut(|i| i.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::Z)) {
                debug!("SHIFT Ctrl-Z");
                if let Some(check_point) = self.redo.pop() {
                    self.undo.push(self.data.clone());
                    self.data = check_point;
                }
            }
        }

        // compute left/right sample
        let mut left: Vec<Pos2> = vec![];
        let mut right: Vec<Pos2> = vec![];

        let step = self.data.len as f32 / height;

        for i in 0..height as usize {
            let t = (((i as f32) * step) as usize + self.data.offset + wav_data.len) % wav_data.len;
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
        if let Some(cursor) = cut_cursor {
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
        if let Some(value) = cut_value {
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
