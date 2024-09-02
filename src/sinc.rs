// sinc interpolation
use crate::cut_panel::Cut;
// use crate::wav_panel::Wav;
// use wav::Header;
// use hound::WavSpec;
use log::debug;

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
    let bpm = 120.0; // in 4/4
    let wav_spec = cut.wav_data.get_wav_spec().unwrap();
    debug!("wav_spec {:?}", wav_spec);

    let _in_rate = wav_spec.sample_rate as f32;
    let out_rate = 48000.0;

    let bars = cut.get_bars();
    debug!("bars {}", bars);

    let out_time_s = bars * 4.0 * 60.0 / bpm;
    debug!("out_time {} seconds", out_time_s);

    let out_samples = out_rate * out_time_s;
    debug!("out_samples {}", out_samples);

    let in_len = cut.wav.get_data_len();
    debug!("in_len {}", in_len);
    let in_out = in_len as f32 / out_samples;
    debug!("in_out {}", in_out);

    let nr_sinc_samples = 10;
    let first_sinc_sample = nr_sinc_samples / 2;

    let _offset = cut.wav.get_data_offset();

    let mut out = vec![];
    for i in 0..out_samples as usize {
        // time in bars
        let t_bars = bars * i as f32 / out_samples;
        // recreate sample at time t
        let t_0_1 = cut.sample_spline(t_bars).unwrap();

        let t = t_0_1 * in_len as f32;

        let min_t = t.floor(); // the sample left of the one to re-create
        let diff = t - min_t;

        let mut left = 0.0;
        let mut right = 0.0;
        for j in 0..=nr_sinc_samples {
            let sample_position = j + min_t as usize - first_sinc_sample;
            let (in_sample_left, in_sample_right) =
                cut.wav.get_sample(sample_position, &cut.wav_data);

            let sinc_sample = sinc((j as f32 - first_sinc_sample as f32) - diff);

            left += sinc_sample * in_sample_left;
            right += sinc_sample * in_sample_right;
        }

        out.push(left);
        out.push(right);
    }

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create("./audio/re_sample.wav", spec).unwrap();
    for s in out.into_iter() {
        let _ = writer.write_sample(s);
    }
    writer.finalize().unwrap();
}
