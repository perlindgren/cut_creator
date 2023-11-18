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
    let fr = 10_000.0;

    let mut sample = vec![];
    for t in (0..sample_rate * time).map(|x| x as f32 / sample_rate as f32) {
        sample.push(attenuation * (t * fr * 2.0 * PI).sin());
    }

    // let mut re_sample = vec![];

    let ratio: f32 = 2.0;
    let dt = 5.0;
    let s = if ratio < 1.0 { 1.0 } else { 1.0 / ratio };

    // sinc stretch factor
    let stretch = (dt / s).floor() as i32;
    let x1 = -stretch;
    let x2 = stretch;
    println!("s {} {}..{}", s, x1, x2);

    let mut n = 0.0;

    for x in x1..=x2 {
        // ts=0.9*(t(k1)-t2(k2) )/s;   %time of each input sample relative to output sample time, stretched to make filter roll off slighty earlier to suppress folding from tones close to fs/2
        let ts = x as f32 * ratio / s;
        // w1=sinc(ts);		%low-pass filter
        let w1 = sinc(ts);
        // w2=(1+cos( pi* ts/dt) );	%window function (hann)
        let w2 = 1.0 + (PI * ts / dt).cos();
        println!("x {}, ts {}, w1 {}, w2 {}", x, ts, w1, w2);

        // d=d+u(k1)*w1*w2;
        // n=n+w1*w2;
        n += w1 * w2;
    }

    println!("n {}", n);

    // println!("first_sinc_sample {}", first_sinc_sample);
    // println!("ratio {}", ratio);
    // println!("stretch {}", stretch);
    // for i in 0..sample_rate * time {
    //     // ratio *= 1.0001;
    //     // recreate sample at time t
    //     let t = i as f32 * ratio;
    //     let t_round = t.round() as usize;

    //     let mut s = 0.0;
    //     let mut w_acc = 0.0;
    //     for j in 0..=nr_sinc_samples {
    //         if let Some(in_sample) = sample.get(j + t_round - first_sinc_sample) {
    //             let ts = (j as f32 - first_sinc_sample as f32) * stretch / ratio;
    //             let w_sinc = sinc(ts);
    //             let w_hann = 1.0 + (PI * ratio * stretch).cos();

    //             let w = w_sinc * w_hann;
    //             w_acc += w;
    //             s += w_acc * in_sample;
    //         }
    //     }
    //     re_sample.push(s / w_acc);
    // }

    // let spec = hound::WavSpec {
    //     channels: 2,
    //     sample_rate: 48000,
    //     bits_per_sample: 32,
    //     sample_format: hound::SampleFormat::Float,
    // };

    // let mut writer = hound::WavWriter::create("./audio/sinc_out.wav", spec).unwrap();
    // for (s1, s2) in sample.clone().iter().zip(re_sample.clone()) {
    //     writer.write_sample(*s1).unwrap();
    //     writer.write_sample(s2).unwrap();
    // }
    // writer.finalize().unwrap();

    // let spec = hound::WavSpec {
    //     channels: 1,
    //     sample_rate: 48000,
    //     bits_per_sample: 32,
    //     sample_format: hound::SampleFormat::Float,
    // };
    // let mut writer = hound::WavWriter::create("./audio/sinc_re.wav", spec).unwrap();
    // for s in re_sample {
    //     writer.write_sample(s).unwrap();
    // }
    // writer.finalize().unwrap();
}
