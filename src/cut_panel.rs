use egui::*;
use egui::{emath::RectTransform, epaint::PathShape};
use epaint::RectShape;
use splines::{Interpolation, Key, Spline};

#[derive(Copy, Clone, Debug)]
pub struct Knot {
    /// x position in terms of bars. 0.25 -> 1st quarter in 1st bar
    /// y position in terms of relative sample position 0.0 beginning of sample 1.0 end of sample.
    pos: Pos2,
    selected: bool,
}

impl Knot {
    fn knot_round(&self, trans: RectTransform) -> Knot {
        unimplemented!()
    }
}

pub struct Cut {
    /// Quantization 4 -> 1/4 = 0.25 (quarter notes), 16-> 1/16 (six teens), etc.
    quantization: u32,

    /// Length in terms of bars, e.g. 1 amounts to 4 quarter notes, etc.
    bars: u32,

    /// The control points.
    knots: Vec<Knot>,

    /// Stroke selected.
    stroke_default: Stroke,

    /// Stroke selected.
    stroke_selected: Stroke,

    /// Select rect
    select_start: Pos2,

    /// Select end
    select_end: Pos2,

    /// Select drag
    select_drag: bool,

    /// Move drag
    move_drag: bool,

    /// Pos for the start of moving points
    move_start: Pos2,

    /// Last position when moving
    move_last: Pos2,

    /// Start positions for each knot
    move_knots: Vec<Pos2>,

    /// Stroke for auxiliary lines.
    stroke_line: Stroke,

    /// Stroke for splines.
    stroke_spline: Stroke,

    /// Stroke grid
    stroke_grid_16: Stroke,

    /// Stroke grid
    stroke_grid_4: Stroke,

    /// Stroke grid
    stroke_grid_1: Stroke,

    /// Spline
    spline: Spline<f32, f32>,

    /// Cursor
    cursor: Pos2,
    // /// drag
    // knot_drag: Option<Pos>
}

impl Cut {
    const SPACE: f32 = 1.0;

    // call to update spline when knots are changed
    fn update(&mut self) {
        self.spline = Spline::from_iter(
            self.knots
                .iter()
                .map(|p| Key::new(p.pos.x, p.pos.y, Interpolation::CatmullRom)),
        );
    }

    /// get the cursor position
    pub fn cursor(&self) -> Pos2 {
        self.cursor
    }
}

impl Default for Cut {
    fn default() -> Self {
        let knots = vec![
            // start top left
            Knot {
                pos: pos2(0.0, 0.0),
                selected: false,
            },
            Knot {
                pos: pos2(1.0, 0.25),
                selected: false,
            },
            Knot {
                pos: pos2(2.0, 0.5),
                selected: false,
            },
            Knot {
                pos: pos2(3.0, 0.75),
                selected: false,
            },
            // end bottom right
            Knot {
                pos: pos2(4.0, 1.0),
                selected: false,
            },
        ];
        let spline = Spline::from_iter(
            knots
                .iter()
                .map(|k| Key::new(k.pos.x, k.pos.y, Interpolation::CatmullRom)),
        );

        Self {
            quantization: 16,
            bars: 4,
            knots,
            stroke_default: Stroke::new(1.0, Color32::WHITE.linear_multiply(0.25)),
            stroke_selected: Stroke::new(1.0, Color32::WHITE),
            select_start: Pos2::ZERO,
            select_end: Pos2::ZERO,
            select_drag: false,
            move_drag: false,
            move_start: Pos2::ZERO,
            move_last: Pos2::ZERO,
            move_knots: vec![],
            stroke_line: Stroke::new(1.0, Color32::RED.linear_multiply(0.25)),
            stroke_spline: Stroke::new(1.0, Color32::BLUE.linear_multiply(1.0)),
            stroke_grid_16: Stroke::new(1.0, Color32::GRAY.linear_multiply(0.01)),
            stroke_grid_4: Stroke::new(2.0, Color32::GRAY.linear_multiply(0.10)),
            stroke_grid_1: Stroke::new(2.0, Color32::GRAY.linear_multiply(0.20)),
            spline,
            cursor: Pos2::ZERO,
        }
    }
}

impl Cut {
    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), ui.available_height()),
            Sense::click_and_drag(),
        );

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        // panel_pos relation to bars
        let width = response.rect.width();
        let segments = self.bars * self.quantization;
        let scale = width / (segments as f32);

        let bars_rect = Rect::from_min_max(
            Pos2::ZERO,
            Pos2 {
                x: self.bars as f32,
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
                !(k.selected && index > 1 && index < len)
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

            self.knots.iter_mut().enumerate().for_each(|(i, k)| {
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
                self.stroke_default,
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
            let scr_pos = response.interact_pointer_pos().unwrap();
            update = true;
            let delta = scr_pos - self.move_last;
            self.move_last = scr_pos;

            let rel = scr_pos - self.move_start;
            // let rel = Pos2 { x: rel.x, y: rel.y };
            let bar_rel = bars_to_screen.inverse().scale() * rel;

            println!("rel {:?}, k rel {:?}", rel, bar_rel);

            if delta.x > 0.0 {
                println!("right");
                // right. we have to update rightmost knot first
                for i in (0..cp.len()).rev() {
                    if self.knots[i].selected {
                        println!("i {} ", i);
                        self.knots[i].pos.y = self.move_knots[i].y + bar_rel.y;
                        // self.knots.pos.x
                        //                 self.knots[i].pos += delta;
                        //                 if i < cp.len() - 1 {
                        //                     self.knots[i].pos.x = self.knots[i]
                        //                         .pos
                        //                         .x
                        //                         .min(self.knots[i + 1].pos.x - Cut::SPACE);
                        //                 }
                        //                 self.knots[i].pos = to_screen.from().clamp(self.knots[i].pos);
                    }
                }
            } else {
                println!("left");
                // left or up/down, we update leftmost knot first
                for i in 0..cp.len() {
                    if self.knots[i].selected {
                        println!("i {} ", i);
                        self.knots[i].pos.y = self.move_knots[i].y + bar_rel.y;
                        //                 self.knots[i].pos += delta;
                        //                 if i > 0 {
                        //                     self.knots[i].pos.x = self.knots[i]
                        //                         .pos
                        //                         .x
                        //                         .max(self.knots[i - 1].pos.x + Cut::SPACE);
                        //                 }
                        //                 self.knots[i].pos = to_screen.from().clamp(self.knots[i].pos);
                    }
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
                let mut point_in_screen = bars_to_screen * k.pos;
                // println!("k {:?}, point in screen {:?}", k, point_in_screen);

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
                    // let delta = point_response.drag_delta();
                    let pos = point_response.interact_pointer_pos().unwrap();
                    let mut knot_pos = bars_to_screen.inverse().transform_pos(pos);
                    println!("single_knot_drag {:?}", knot_pos.x);
                    knot_pos.x = (knot_pos.x * (self.quantization as f32)).round()
                        / (self.quantization as f32);
                    println!("rounded {:?}", knot_pos.x);

                    // never move first and last knots in x direction
                    if i > 0 && i < cp.len() - 1 {
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
                        self.stroke_selected
                    } else {
                        self.stroke_default
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
        if self.knots.len() > 3 {
            let start = self.knots[1].pos.x; // to ensure we have two knots on either side
            let end = self.knots[self.knots.len() - 2].pos.x;

            let interval = end - start;
            let points: u32 = 1000;
            let step = interval / (points as f32);

            let mut v = vec![];

            for i in 0..points {
                let t = i as f32 * step + start;
                v.push(
                    bars_to_screen
                        * Pos2 {
                            x: t,
                            y: self.spline.sample(t).unwrap(),
                        },
                )
            }

            painter.add(PathShape::line(v, self.stroke_spline));
        }

        // draw connecting lines for spline
        let points_in_screen: Vec<Pos2> =
            self.knots.iter().map(|k| bars_to_screen * k.pos).collect();
        painter.add(PathShape::line(points_in_screen, self.stroke_line));

        // knots
        painter.extend(control_point_shapes);

        // hover line
        if let Some(pos) = ui
            .interact(response.rect, ui.id(), Sense::hover())
            .hover_pos()
        {
            self.cursor = pos;

            let logic_pos = to_screen.inverse().transform_pos(pos);
            let segment_pos = logic_pos.x / scale;
            // println!(
            //     "pos {:?}, logic_pos {:?}, segment_pos {:?}",
            //     pos, logic_pos, segment_pos
            // );
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
                self.stroke_default,
            ));
        }

        // grid
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
                    self.stroke_grid_1
                } else if t % 4 == 0 {
                    self.stroke_grid_4
                } else {
                    self.stroke_grid_16
                },
            ));
        }

        response
    }
}
