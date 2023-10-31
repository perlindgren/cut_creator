// % sinc(x) = sin(pi*x)/(pi*x), sinc(0)=1
// step=.1; % cutoff is f = step*Fs/2
// limit=step*40; % increase to reduce ripples
// c=[]; i=1;
// for x = -limit : step : +limit,
// if abs(x)>0,c(i)=sin(pi*x)/(pi*x);
// else
//  c(i)=1;
// end;
// i=i+1;
// end;

use std::f32::consts::PI;

fn sinc(x: f32) -> f32 {
    if x != 0.0 {
        (x * PI).sin() / (x * PI)
    } else {
        1.0
    }
}

fn sinc_fir(l: usize) -> Vec<f32> {
    // let mut fir = vec![];
    let fs = 48000.0;
    let step = 0.1;
    let cutoff = step * fs / 2.0;
    let limit = 10;
    for x in 0..limit {
        println!("{}", x);
    }
    unimplemented!()
}
fn main() {
    let _ = sinc_fir(0);
}
