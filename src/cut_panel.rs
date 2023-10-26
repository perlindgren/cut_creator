use crate::{
    config::Config,
    wav_panel::{Wav, WavData},
};
use egui::epaint::PathShape;
use egui::*;
use epaint::RectShape;
use splines::{Interpolation, Spline};

use serde::{Deserialize, Serialize};
use std::{fs::File, io::prelude::*, path::PathBuf};

use log::trace;
/// cut_panel
///
/// A cut is defined by a spline with CatmullRom interpolation.
/// Left and right knots are outside of the cut region by 1/4 bar.
/// Second left endpoint defines S the start position of the sample.
/// Second right endpoint defines E the end position of the sample.
///
/// The loop option forces S <-> E, which ensures that the cut can be smoothly looped.

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Knot {
    /// x position in terms of bars. 0.25 -> 1st quarter in 1st bar
    /// y position in terms of relative sample position 0.0 beginning of sample 1.0 end of sample.
    pos: Pos2,
    selected: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cut {
    /// path to the cut
    pub path: PathBuf,

    /// path to the sample
    pub sample_path: Option<PathBuf>,

    /// Quantization 4 -> 1/4 = 0.25 (quarter notes), 16-> 1/16 (six teens), etc.
    quantization: u32,

    /// Length in terms of bars, e.g. 1.0 amounts to 4 quarter notes, etc.
    bars: f32,

    /// The control points.
    knots: Vec<Knot>,

    /// Looping, the end point equates the start point
    looping: bool,

    /// Warping, the samples will warp across start/end
    warping: bool,

    /// Spline
    // TODO, should this be #[serde(Skip)]? (Default impl required)
    spline: Spline<f32, f32>,

    /// Wav
    pub wav: Wav,

    /// Run-time only data

    /// Needs save
    #[serde(skip)]
    pub needs_save: bool,

    /// Wav Data
    #[serde(skip)]
    pub wav_data: WavData,

    /// Select rect
    #[serde(skip)]
    select_start: Pos2,

    /// Select end
    #[serde(skip)]
    select_end: Pos2,

    /// Select drag
    #[serde(skip)]
    select_drag: bool,

    /// Move drag
    #[serde(skip)]
    move_drag: bool,

    /// Pos for the start of moving points
    #[serde(skip)]
    move_start: Pos2,

    /// Last position when moving
    #[serde(skip)]
    move_last: Pos2,

    /// Start positions for each knot
    #[serde(skip)]
    move_knots: Vec<Pos2>,

    /// Cursor
    #[serde(skip)]
    cursor: Option<Pos2>,

    /// Value
    #[serde(skip)]
    value: Option<f32>,
}

impl Default for Cut {
    fn default() -> Self {
        let knots = vec![
            // start top left
            Knot {
                pos: pos2(-0.25, 0.0),
                selected: false,
            },
            Knot {
                pos: pos2(0.0, 0.0),
                selected: false,
            },
            Knot {
                pos: pos2(1.0, 0.5),
                selected: false,
            },
            Knot {
                pos: pos2(2.0, 1.0),
                selected: false,
            },
            // end top right
            Knot {
                pos: pos2(2.25, 1.0),
                selected: false,
            },
        ];
        let spline = Spline::from_iter(
            knots
                .iter()
                .map(|k| splines::Key::new(k.pos.x, k.pos.y, Interpolation::CatmullRom)),
        );

        Self {
            path: PathBuf::new(),
            sample_path: None,
            quantization: 16,
            bars: 2.0,
            knots,
            spline,
            wav: Wav::default(),
            wav_data: WavData::default(),

            // Non persistent data
            needs_save: false,
            select_start: Pos2::ZERO,
            select_end: Pos2::ZERO,
            select_drag: false,
            move_drag: false,
            move_start: Pos2::ZERO,
            move_last: Pos2::ZERO,
            move_knots: vec![],
            cursor: None,
            value: None,
            looping: false,
            warping: false,
        }
    }
}

impl Cut {
    pub fn load_wav() -> Result<Cut, String> {
        match rfd::FileDialog::new()
            .add_filter("wav", &["wav", "cut"])
            .set_directory("./audio/")
            .pick_file()
        {
            Some(mut path) => {
                trace!("path {:?}", path);

                match path.extension() {
                    Some(ext) => {
                        trace!("ext {:?}", ext);
                        match ext.to_str() {
                            Some("wav") => {
                                // loading wav only, set the cut to default
                                trace!("load wav");
                                let mut cut = Cut::default();
                                cut.wav_data = WavData::load_wav_data(path.clone());
                                cut.wav.len = cut.wav_data.len;
                                println!("path {}", path.display());
                                cut.sample_path = Some(path.clone());
                                path.set_extension("cut");
                                cut.path = path;
                                Ok(cut)
                            }
                            Some("cut") => {
                                trace!("load cut");
                                if let Ok(mut file) = File::open(path) {
                                    let mut json = String::new();
                                    file.read_to_string(&mut json).unwrap();
                                    trace!("json {}", json);
                                    let mut cut: Cut = serde_json::from_str(&json).unwrap();
                                    trace!("cut {:?}", cut);

                                    if let Some(sample_path) = cut.sample_path.clone() {
                                        cut.wav_data = WavData::load_wav_data(sample_path);
                                    }
                                    Ok(cut)
                                } else {
                                    Err("Could not load file".to_string())
                                }
                            }

                            _ => Err("Filetype not supported".to_string()),
                        }
                    }
                    _ => Err("Filetype not supported".to_string()),
                }
            }
            None => Err("Load cancelled".to_string()),
        }
    }

    /// name
    pub fn name(&self) -> String {
        self.path.file_name().map_or("<TBD>".to_string(), |path| {
            path.to_string_lossy().to_string()
        })
    }

    /// load cut
    fn _load_cut() -> Result<Self, ()> {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("cut", &["cut"])
            .set_directory("./audio/")
            .pick_file()
        {
            trace!("load cut path {:?}", path);
        }
        Err(())
    }

    /// call to update spline when knots are changed
    pub fn update(&mut self) {
        trace!("update knots and spline");
        self.needs_save = true;
        let len = self.knots.len();
        // ensure that endpoints are aligned
        self.knots[0].pos.y = self.knots[1].pos.y;

        if self.looping {
            self.knots[len - 2].pos.y = self.knots[0].pos.y;
            self.knots[len - 1].pos.y = self.knots[len - 2].pos.y;
        } else {
            self.knots[len - 1].pos.y = self.knots[len - 2].pos.y;
        }
        self.spline = Spline::from_iter(
            self.knots
                .iter()
                .map(|k| splines::Key::new(k.pos.x, k.pos.y, Interpolation::CatmullRom)),
        );
    }

    /// get the cursor position
    pub fn get_cursor(&self) -> Option<Pos2> {
        self.cursor
    }

    /// get value at cursor position
    pub fn get_value(&self) -> Option<f32> {
        self.value
    }

    /// save cut
    fn save_cut(&mut self) -> String {
        // Serialize it to a JSON string.
        let json = serde_json::to_string(&self).unwrap();
        trace!("cut json {}", json);

        match File::create(&self.path) {
            Ok(mut file) => {
                self.needs_save = false;
                if let Err(err) = file.write_all(json.as_bytes()) {
                    println!("Err {:?}", err);
                    format!("{:?}", err)
                } else {
                    format!("Cut saved to {:?}", self.path)
                }
            }
            Err(err) => {
                format!("{:?}", err)
            }
        }
    }

    ///
    fn save_cut_dialogue(&mut self) -> String {
        let directory = self.path.parent().unwrap();
        trace!("directory : {:?}", directory);
        let file_name = self.path.file_name().unwrap().to_string_lossy();
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("cut", &["cut"])
            .set_directory(directory)
            .set_file_name(file_name)
            .save_file()
        {
            trace!("cut path {:?}", path);
            self.path = path;
            self.save_cut()
        } else {
            "Save cancelled.".to_string()
        }
    }

    /// Cut Settings
    pub fn ui_content_settings(&mut self, ui: &mut Ui, status: &mut String) {
        if ui.checkbox(&mut self.looping, "looping").clicked()
            || ui.checkbox(&mut self.warping, "warping").clicked()
        {
            self.update()
        }

        if ui
            .button("Save Cut")
            // TODO
            .clicked()
            || ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::S))
        {
            *status = self.save_cut();
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::S)) {
            *status = self.save_cut_dialogue();
        }
    }

    /// main panel
    pub fn ui_content(&mut self, ui: &mut Ui, config: &Config, height: f32) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            // Vec2::new(ui.available_width(), ui.available_height()),
            Vec2::new(ui.available_width(), height),
            Sense::click_and_drag(),
        );

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        // panel_pos relation to bars
        let width = response.rect.width();
        let segments = self.bars * self.quantization as f32;
        let scale = width / segments;

        let bars_rect = Rect::from_min_max(
            Pos2::ZERO,
            Pos2 {
                x: self.bars,
                y: 1.0,
            },
        );

        let bars_to_screen = emath::RectTransform::from_to(bars_rect, response.rect);

        let mut clicked = response.clicked();
        let mut update = false;

        if ui.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl) {
            println!("undo");
        }

        // delete knot
        if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
            println!("delete");
            let mut index = 0;
            let len = self.knots.len();

            self.knots.retain(|k| {
                index += 1;
                !(k.selected && index > 2 && index < len - 1)
            });

            update = true;
        }

        // selection
        // unselect all by ESCAPE or double right click
        if ui.input(|i| i.key_pressed(egui::Key::Escape))
            || response.double_clicked_by(PointerButton::Secondary)
        {
            println!("escape");
            for k in self.knots.iter_mut() {
                k.selected = false;
            }
        }

        if response.drag_started_by(PointerButton::Secondary) {
            let pos = response.interact_pointer_pos().unwrap();
            println!("select start {:?}", pos);
            self.select_start = pos;
            self.select_end = pos;
            self.select_drag = true;
        }

        if response.drag_released_by(PointerButton::Secondary) {
            let pos = response.interact_pointer_pos().unwrap();
            println!("select end {:?} ", pos);
            let rect = Rect::from_two_pos(self.select_start, self.select_end);

            self.knots.iter_mut().for_each(|k| {
                if rect.contains(bars_to_screen * k.pos) {
                    k.selected ^= true;
                }
            });

            update = true;
            self.select_drag = false;
        }

        if response.dragged_by(PointerButton::Secondary) {
            let pos = response.interact_pointer_pos().unwrap();
            println!("select changed {:?}", pos);

            self.select_end = pos;
        }

        // paint selection rectangle
        if self.select_drag {
            painter.add(Shape::Rect(RectShape::stroke(
                Rect::from_two_pos(self.select_start, self.select_end),
                Rounding::default(),
                config.stroke_select,
            )));
        }

        let cp = self.knots.clone();
        // drag all knots
        if response.drag_started_by(PointerButton::Primary) {
            self.move_drag = true;
            self.move_start = response.interact_pointer_pos().unwrap();
            self.move_last = self.move_start;
            self.move_knots = self.knots.iter().map(|k| k.pos).collect();
            println!("start move {:?}", self.move_start);
        }

        if response.drag_released_by(PointerButton::Primary) {
            self.move_drag = true;
            println!("end move");
            self.move_drag = false;
        }

        if response.dragged_by(PointerButton::Primary) {
            update = true;
            let scr_pos = response.interact_pointer_pos().unwrap();
            let delta = scr_pos - self.move_last;
            self.move_last = scr_pos;

            let rel = scr_pos - self.move_start;
            let bar_rel = bars_to_screen.inverse().scale() * rel;

            println!("rel {:?}, k rel {:?}", rel, bar_rel);

            if delta.x > 0.0 {
                println!("right");
                // right. we have to update rightmost knot first
                // exclude first 2 and last 2 knots, they have fixed x positions
                for i in (2..cp.len() - 2).rev() {
                    if self.knots[i].selected {
                        let knot_pos_x = ((self.move_knots[i].x + bar_rel.x)
                            * (self.quantization as f32))
                            .round()
                            / (self.quantization as f32);

                        if knot_pos_x < self.knots[i + 1].pos.x
                            && knot_pos_x > self.knots[i - 1].pos.x
                        {
                            self.knots[i].pos.x = knot_pos_x;
                        }
                    }
                }
            } else if delta.x < 0.0 {
                println!("left");
                // left we update leftmost knot first
                // we exclude first 2 and last 2 knots, they have fixed positions
                for i in 2..cp.len() - 2 {
                    if self.knots[i].selected {
                        println!("i {} ", i);

                        let knot_pos_x = ((self.move_knots[i].x + bar_rel.x)
                            * (self.quantization as f32))
                            .round()
                            / (self.quantization as f32);

                        if knot_pos_x > self.knots[i - 1].pos.x
                            && knot_pos_x < self.knots[i + 1].pos.x
                        {
                            self.knots[i].pos.x = knot_pos_x;
                        }
                    }
                }
            }
            println!("none");
            // left or up/down, we update leftmost knot first
            for i in 0..cp.len() {
                if self.knots[i].selected {
                    println!("i {} ", i);
                    self.knots[i].pos.y = (self.move_knots[i].y + bar_rel.y).min(1.0).max(0.0);
                }
            }
        }

        let control_point_radius = 8.0;
        // knots
        let control_point_shapes: Vec<Shape> = self
            .knots
            .iter_mut()
            .enumerate()
            .map(|(i, k)| {
                let size = Vec2::splat(2.0 * control_point_radius);
                let point_in_screen = bars_to_screen * k.pos;

                let point_rect = Rect::from_center_size(point_in_screen, size);

                let point_id = response.id.with(i);
                let point_click = ui.interact(point_rect, point_id, Sense::click());

                // toggle select on click
                if point_click.clicked() {
                    k.selected ^= true;
                    clicked = false;
                }

                let point_response = ui.interact(point_rect, point_id, Sense::drag());

                if point_response.drag_released() {
                    println!("released");
                }

                if point_response.dragged() {
                    let pos = point_response.interact_pointer_pos().unwrap();
                    let mut knot_pos = bars_to_screen.inverse().transform_pos(pos);
                    println!("single_knot_drag {:?}", knot_pos.x);
                    knot_pos.x = (knot_pos.x * (self.quantization as f32)).round()
                        / (self.quantization as f32);
                    println!("rounded {:?}", knot_pos.x);

                    // never move first 2 and last 2 knots in x direction
                    if i > 1 && i < cp.len() - 2 {
                        if knot_pos.x > k.pos.x {
                            // right
                            if knot_pos.x < cp[i + 1].pos.x {
                                k.pos.x = knot_pos.x;
                            }
                        } else if knot_pos.x < k.pos.x {
                            // left
                            if knot_pos.x > cp[i - 1].pos.x {
                                k.pos.x = knot_pos.x;
                            }
                        }
                    }

                    k.pos.y = knot_pos.y.min(1.0).max(0.0); // clamp to range

                    update = true;
                }

                Shape::circle_stroke(
                    point_in_screen,
                    control_point_radius,
                    if k.selected {
                        config.stroke_knot_selected
                    } else {
                        config.stroke_knot
                    },
                )
            })
            .collect();

        // add new point
        if clicked {
            // screen position
            let click_pos = response.interact_pointer_pos().unwrap();
            // data point
            let mut pos = bars_to_screen.inverse().transform_pos_clamped(click_pos);

            let round_x = (pos.x * self.quantization as f32).round() / (self.quantization as f32);

            pos.x = round_x;

            // insert, or move
            let cp = self.knots.clone().into_iter();

            if self
                .knots
                .iter_mut()
                .find_map(|k| {
                    if k.pos.x == pos.x {
                        k.pos.y = pos.y;
                        Some(())
                    } else {
                        None
                    }
                })
                .is_none()
            {
                println!("new point");
                let (head, mut tail): (Vec<_>, Vec<_>) = cp.partition(|k| pos.x < k.pos.x);

                tail.push(Knot {
                    pos,
                    selected: false,
                });
                tail.extend(head);

                self.knots = tail;

                println!("knots {:?}", self.knots);
            }
            update = true;
        }

        if update {
            self.update();
        }

        // draw spline
        let start = self.knots[1].pos.x; // to ensure we have two knots on either side
        let end = self.knots[self.knots.len() - 2].pos.x;

        let interval = end - start;
        let points = width / config.step_size as f32;
        let step = interval / points;

        let mut v = vec![];

        for i in 0..=points as usize {
            let t = i as f32 * step + start;

            if let Some(y) = self.spline.sample(t) {
                let y = if self.warping {
                    if y > 1.0 {
                        y - 1.0
                    } else if y < 0.0 {
                        y + 1.0
                    } else {
                        y
                    }
                } else {
                    y.max(0.0).min(1.0)
                };

                v.push(bars_to_screen * Pos2 { x: t, y })
            }
        }
        v.push(
            bars_to_screen
                * Pos2 {
                    x: end,
                    y: self.spline.sample(end - 0.000001).unwrap(),
                },
        );

        painter.add(PathShape::line(v, config.stroke_spline));

        // draw connecting lines for spline
        if config.knot_line {
            let points_in_screen: Vec<Pos2> =
                self.knots.iter().map(|k| bars_to_screen * k.pos).collect();
            painter.add(PathShape::line(points_in_screen, config.stroke_line));
        }

        // knots
        painter.extend(control_point_shapes);

        // hover line
        if let Some(pos) = ui
            .interact(response.rect, ui.id(), Sense::hover())
            .hover_pos()
        {
            self.cursor = Some(pos);
            let t = bars_to_screen.inverse().transform_pos(pos).x;

            if let Some(y) = self.spline.sample(t) {
                let y = if self.warping {
                    if y > 1.0 {
                        y - 1.0
                    } else if y < 0.0 {
                        y + 1.0
                    } else {
                        y
                    }
                } else {
                    y.max(0.0).min(1.0)
                };
                self.value = Some(y);
            } else {
                self.value = None;
            }

            // println!("cut ratio {:?}", self.value);

            let logic_pos = to_screen.inverse().transform_pos(pos);
            let segment_pos = logic_pos.x / scale;
            let round_segment_x = segment_pos.round();

            let logic_x = round_segment_x * scale;

            let _segment = painter.add(PathShape::line(
                vec![
                    to_screen * Pos2 { x: logic_x, y: 0.0 },
                    to_screen
                        * Pos2 {
                            x: logic_x,
                            y: response.rect.height(),
                        },
                ],
                config.stroke_line,
            ));
        } else {
            self.cursor = None;
            self.value = None;
        }

        // grid
        let stroke_grid_16 = Stroke::new(1.0, Color32::GRAY.linear_multiply(0.01));
        let stroke_grid_4 = Stroke::new(2.0, Color32::GRAY.linear_multiply(0.10));
        let stroke_grid_1 = Stroke::new(2.0, Color32::GRAY.linear_multiply(0.20));
        for t in 0..segments as usize {
            painter.add(PathShape::line(
                vec![
                    to_screen
                        * Pos2 {
                            x: t as f32 * scale,
                            y: 0.0,
                        },
                    to_screen
                        * Pos2 {
                            x: t as f32 * scale,
                            y: response.rect.height(),
                        },
                ],
                if t % (16) == 0 {
                    stroke_grid_1
                } else if t % 4 == 0 {
                    stroke_grid_4
                } else {
                    stroke_grid_16
                },
            ));
        }

        response
    }
}
