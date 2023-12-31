use egui::{Color32, Stroke};
/// config
///
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    /// Stroke selected.
    pub stroke_select: Stroke,

    /// Stroke knot.
    pub stroke_knot: Stroke,

    /// Stroke knot selected.
    pub stroke_knot_selected: Stroke,

    /// Knot lines
    pub knot_line: bool,

    /// Step size
    pub step_size: u32,

    /// Stroke for auxiliary lines
    pub stroke_line: Stroke,

    /// Stroke for splines.
    pub stroke_spline: Stroke,

    /// Grid color
    pub grid_color: Color32,

    /// Stroke sample
    pub stroke_sample: Stroke,

    /// Strake fader
    pub stroke_fader: Stroke,

    /// Fader height, in pixels
    pub fader_height: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            stroke_select: Stroke::new(1.0, Color32::WHITE.linear_multiply(0.25)),
            stroke_knot: Stroke::new(1.0, Color32::WHITE.linear_multiply(0.25)),
            stroke_knot_selected: Stroke::new(1.0, Color32::WHITE),
            knot_line: true,
            step_size: 10,
            stroke_line: Stroke::new(1.0, Color32::RED.linear_multiply(0.25)),
            grid_color: Color32::GRAY,
            stroke_spline: Stroke::new(1.0, Color32::BLUE.linear_multiply(1.0)),
            stroke_sample: Stroke::new(1.0, Color32::GREEN.linear_multiply(0.25)),
            stroke_fader: Stroke::new(1.0, Color32::YELLOW.linear_multiply(1.0)),
            fader_height: 30.0,
        }
    }
}
