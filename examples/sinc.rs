use std::f32::consts::PI;

// normalized sinc, 0 at 1
fn sinc(x: f32) -> f32 {
    if x != 0.0 {
        (x * PI).sin() / (x * PI)
    } else {
        1.0
    }
}

fn main() {
    let sample_rate = 48000;
    let time = 1;
    let attenuation = 0.5;
    let fr = 440.0;

    let mut sample = vec![];
    for t in (0..sample_rate * time).map(|x| x as f32 / sample_rate as f32) {
        sample.push(attenuation * (t * fr * 2.0 * PI).sin());
    }

    let mut re_sample = vec![];

    let nr_sinc_samples = 11;
    let first_sinc_sample = (nr_sinc_samples - 1) / 2;
    println!("{}", first_sinc_sample);

    let ratio = 0.01;
    for i in 0..sample_rate * time {
        // ratio *= 1.0001;
        // recreate sample at time t
        let t = i as f32 * ratio;
        let min_t = t.floor(); // the sample left of the one to re-create
        let diff = t - min_t;

        let mut s = 0.0;
        for j in 0..=nr_sinc_samples {
            let sample_position = j - first_sinc_sample + min_t as usize;
            if let Some(in_sample) = sample.get(sample_position) {
                s += sinc((j as f32 - first_sinc_sample as f32) - diff) * in_sample;
            }
        }
        re_sample.push(s);
    }

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create("./audio/sinc_out.wav", spec).unwrap();
    for (s1, s2) in sample.clone().iter().zip(re_sample) {
        writer.write_sample(*s1).unwrap();
        writer.write_sample(s2).unwrap();
    }
    writer.finalize().unwrap();
}
