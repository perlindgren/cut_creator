use std::f32::consts::PI;

use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

fn main() {
    let sample_rate = 48000;
    let time = 1;
    let attenuation = 0.5;

    // generate sample
    let mut sample = vec![];
    for t in (0..sample_rate * time).map(|x| x as f32 / sample_rate as f32) {
        sample.push(attenuation * (t * 440.0 * 2.0 * PI).sin());
    }

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create("./audio/sine.wav", spec).unwrap();
    for s in sample.clone() {
        writer.write_sample(s).unwrap();
    }

    // resample
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };
    let mut resampler = SincFixedIn::<f32>::new(96000.0 / 48000.0, 10.0, params, 1024, 1).unwrap();

    let sample_vec = vec![sample];
    let mut resample = resampler.process(&sample_vec, None).unwrap();
    println!("resample len {}", resample.len());
    let resample = resample.pop().unwrap();

    for s in resample {
        writer.write_sample(s).unwrap();
    }

    writer.finalize().unwrap();
}
