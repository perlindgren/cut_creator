// sinc interpolation
use crate::cut_panel::Cut;
use crate::wav_panel::Wav;
use wav::Header;

use std::f32::consts::PI;

#[inline(always)]
fn sinc(x: f32) -> f32 {
    if x != 0.0 {
        (x * PI).sin() / (x * PI)
    } else {
        1.0
    }
}

pub fn sinc_resample(cut: &Cut) {
    println!("here");

    let bpm = 60.0; // in 4/4
    let header = cut.wav_data.get_header();
    println!("header {:?}", header);
}
