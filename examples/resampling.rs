use std::f32::consts::PI;

fn main() {
    let sample_rate = 48000;
    let time = 2;
    let attenuation = 0.5;

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create("./audio/sine.wav", spec).unwrap();
    for t in (0..sample_rate * time).map(|x| x as f32 / sample_rate as f32) {
        let sample = attenuation * (t * 440.0 * 2.0 * PI).sin();

        writer.write_sample(sample).unwrap();
        writer.write_sample(-sample).unwrap();
    }

    writer.finalize().unwrap();
}
