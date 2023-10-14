use egui::epaint::PathShape;
use egui::*;
use std::fs::File;
use std::path::Path;
use wav::{BitDepth, Header};

pub struct Wav {
    stroke_default: Stroke,
    stroke_sample: Stroke,
    header: Header,
    stereo: Vec<f32>,
    left: Vec<f32>,
    right: Vec<f32>,
}

impl Default for Wav {
    fn default() -> Self {
        let mut inp_file = File::open(Path::new("audio/ahh.wav")).unwrap();
        let (header, data) = wav::read(&mut inp_file).unwrap();
        println!("header {:?}", header);

        let stereo = match data {
            BitDepth::ThirtyTwoFloat(v) => {
                println!("len {}", v.len());
                v
            }
            _ => {
                vec![]
            }
        };

        let mut v = stereo.iter();
        let mut left = vec![];
        let mut right = vec![];
        while let Some(l) = v.next() {
            left.push(*l);
            right.push(*v.next().unwrap())
        }

        Self {
            stroke_default: Stroke::new(1.0, Color32::WHITE),
            stroke_sample: Stroke::new(1.0, Color32::GREEN.linear_multiply(0.25)),
            header,
            stereo,
            left,
            right,
        }
    }
}

impl Wav {
    pub fn ui_content(&mut self, ui: &mut Ui, cursor: Pos2) -> egui::Response {
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

        // compute left/right sample
        let mut left: Vec<Pos2> = vec![];
        let mut right: Vec<Pos2> = vec![];
        let len = self.left.len();
        let step = len as f32 / height;

        for i in 0..height as usize {
            let t = ((i as f32) * step) as usize;

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

        response
    }
}
