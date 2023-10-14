use egui::epaint::PathShape;
use egui::*;
use epaint::RectShape;
use splines::{Interpolation, Key, Spline};

#[derive(Copy, Clone, Debug)]
pub struct Knot {
    pos: Pos2,
    selected: bool,
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
            Knot {
                pos: pos2(0.0, 0.0),
                selected: false,
            },
            Knot {
                pos: pos2(400.0, 0.0),
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

        let mut clicked = response.clicked();
        let mut update = false;

        if ui.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl) {
            println!("undo");
        }

        // delete knot
        if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
            println!("delete");

            self.knots.retain(|k| !k.selected);

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
            println!("dragged_start {:?} ", pos);
            self.select_start = pos;
            self.select_end = pos;
            self.select_drag = true;
        }

        if response.drag_released_by(PointerButton::Secondary) {
            let pos = response.interact_pointer_pos().unwrap();
            println!("dragged_release {:?} ", pos);
            let rect = Rect::from_two_pos(self.select_start, self.select_end);

            self.knots.iter_mut().enumerate().for_each(|(i, k)| {
                if rect.contains(to_screen * k.pos) {
                    k.selected ^= true;
                }
            });

            update = true;
            self.select_drag = false;
        }

        if response.dragged_by(PointerButton::Secondary) {
            let pos = response.interact_pointer_pos().unwrap();
            println!("dragged_by {:?}", pos);

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
        if response.dragged_by(PointerButton::Primary) {
            let delta = response.drag_delta();
            println!("dragged {:?}", response.drag_delta());
            update = true;

            if delta.x > 0.0 {
                // right. we have to update rightmost knot first
                for i in (0..cp.len()).rev() {
                    if self.knots[i].selected {
                        self.knots[i].pos += delta;
                        if i < cp.len() - 1 {
                            self.knots[i].pos.x = self.knots[i]
                                .pos
                                .x
                                .min(self.knots[i + 1].pos.x - Cut::SPACE);
                        }
                        self.knots[i].pos = to_screen.from().clamp(self.knots[i].pos);
                    }
                }
            } else {
                // left or up/down, we update leftmost knot first
                for i in 0..cp.len() {
                    if self.knots[i].selected {
                        self.knots[i].pos += delta;
                        if i > 0 {
                            self.knots[i].pos.x = self.knots[i]
                                .pos
                                .x
                                .max(self.knots[i - 1].pos.x + Cut::SPACE);
                        }
                        self.knots[i].pos = to_screen.from().clamp(self.knots[i].pos);
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

                let point_in_screen = to_screen.transform_pos(k.pos);
                let point_rect = Rect::from_center_size(point_in_screen, size);

                let point_id = response.id.with(i);
                let point_click = ui.interact(point_rect, point_id, Sense::click());

                // toggle select on click
                if point_click.clicked() {
                    k.selected ^= true;
                    clicked = false;
                }

                let point_response = ui.interact(point_rect, point_id, Sense::drag());
                let delta = point_response.drag_delta();

                if delta != Vec2::ZERO {
                    println!("----");
                    update = true;

                    k.pos += delta;
                    if delta.x > 0.0 {
                        // right
                        if i < cp.len() - 1 {
                            k.pos.x = k.pos.x.min(cp[i + 1].pos.x - Cut::SPACE);
                        }
                    } else if delta.x < 0.0 {
                        // left
                        if i > 0 {
                            k.pos.x = k.pos.x.max(cp[i - 1].pos.x + Cut::SPACE);
                        }
                    }
                    k.pos = to_screen.from().clamp(k.pos);
                }

                let point_in_screen = to_screen.transform_pos(k.pos);

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
            let pos = to_screen.inverse().transform_pos_clamped(click_pos);

            // insert
            let cp = self.knots.clone().into_iter();

            let (head, mut tail): (Vec<_>, Vec<_>) = cp.partition(|k| pos.x < k.pos.x - Cut::SPACE);

            if let Some(tail_fst) = tail.first() {
                println!("t fst {:?}", tail_fst);
            }

            if let Some(tail_last) = tail.last() {
                println!("t last {:?}", tail_last);
            }

            if let Some(head_fst) = head.first() {
                println!("h fst {:?}", head_fst);
                if head_fst.pos.x < pos.x + 2.0 * Cut::SPACE {
                    println!("!!!!!!!!!!!");
                }
            }

            if let Some(head_last) = head.last() {
                println!("h last  {:?}", head_last);
            }

            tail.push(Knot {
                pos,
                selected: false,
            });
            tail.extend(head);

            self.knots = tail;
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
                    to_screen
                        * Pos2 {
                            x: t,
                            y: self.spline.sample(t).unwrap(),
                        },
                )
            }

            painter.add(PathShape::line(v, self.stroke_spline));
        }

        // draw connecting lines for spline
        let points_in_screen: Vec<Pos2> = self.knots.iter().map(|k| to_screen * k.pos).collect();
        painter.add(PathShape::line(points_in_screen, self.stroke_line));

        painter.extend(control_point_shapes);

        if let Some(pos) = ui
            .interact(response.rect, ui.id(), Sense::hover())
            .hover_pos()
        {
            self.cursor = pos;
            painter.add(PathShape::line(
                vec![
                    Pos2 {
                        x: pos.x,
                        y: response.rect.top(),
                    },
                    Pos2 {
                        x: pos.x,
                        y: response.rect.bottom(),
                    },
                ],
                self.stroke_default,
            ));
        }

        // grid

        let width = response.rect.width();
        let segments = self.bars * self.quantization;
        let scale = width / (segments as f32);

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
