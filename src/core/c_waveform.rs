#![allow(unused)]

use std::f64::consts::{PI, TAU};
use crate::core::c_pitch::Note;

pub enum Waveform {
    Sine, Sawtooth, Square, Triangle, Pulse(i32 /* Pulsewidth */),
    // PinkN, BrownN, WhiteN, VelvetN(i32 /* Density */), BlueN, GreyN, VioletN, GreenN,
    ReverseSawtooth, HalfRectSine, FullRectSine,
    ImpulseTrain, Staircase(i32 /* Steps */), SineCubed,
    GaussianPulse(i32 /* Width */), Trapezoid(i32 /* Flat-top width */), Semicircle, Parabolic,
    SincPulse(i32 /* Lobes */), CauchyPulse(i32 /* Width */), ExponentialSaw(i32 /* Curvature */), Doublet,
    WavefoldedSine(i32 /* Fold amount */), SoftClippedSine(i32 /* Drive */), SelfFMSine(i32 /* Feedback */),
    Chebyshev(u8 /* Harmonic Multiplier */),
    Weierstrass(u8 /* Terms */), Takagi(u8 /* Terms */), Cantor(u8 /* Depth */), ThueMorse(u8 /* Bits */),
}

fn phase_at(note: Note, time: f64) -> f64 {
    (note.frequency_hz() * time).rem_euclid(1.0)
}

pub fn sine(note: Note, time: f64) -> f64 {
    (TAU * note.frequency_hz() * time).sin()
}

pub fn sawtooth(note: Note, time: f64) -> f64 {
    2.0 * phase_at(note, time) - 1.0
}

pub fn square(note: Note, time: f64) -> f64 {
    if phase_at(note, time) < 0.5 { 1.0 } else { -1.0 }
}

pub fn triangle(note: Note, time: f64) -> f64 {
    1.0 - 4.0 * (phase_at(note, time) - 0.5).abs()
}

pub fn pulse(note: Note, time: f64, width: i32) -> f64 {
    let duty = width.clamp(0, 100) as f64 / 100.0;
    if phase_at(note, time) < duty { 1.0 } else { -1.0 }
}

pub fn reverse_sawtooth(note: Note, time: f64) -> f64 {
    1.0 - 2.0 * phase_at(note, time)
}

pub fn half_rect_sine(note: Note, time: f64) -> f64 {
    sine(note, time).max(0.0)
}

pub fn full_rect_sine(note: Note, time: f64) -> f64 {
    sine(note, time).abs()
}

pub fn impulse_train(note: Note, time: f64) -> f64 {
    if phase_at(note, time) < 0.03 { 1.0 } else { 0.0 }
}

pub fn staircase(note: Note, time: f64, steps: i32) -> f64 {
    let n = steps.clamp(2, 256) as f64;
    let level = (phase_at(note, time) * n).floor().min(n - 1.0);
    2.0 * level / (n - 1.0) - 1.0
}

pub fn sine_cubed(note: Note, time: f64) -> f64 {
    sine(note, time).powi(3)
}

pub fn gaussian_pulse(note: Note, time: f64, width: i32) -> f64 {
    let sigma = width.clamp(1, 100) as f64 / 100.0 * 0.25;
    let x = (phase_at(note, time) - 0.5) / sigma;
    (-0.5 * x * x).exp()
}

pub fn trapezoid(note: Note, time: f64, flat_top: i32) -> f64 {
    let flat = flat_top.clamp(0, 99) as f64 / 100.0;
    (triangle(note, time) / (1.0 - flat)).clamp(-1.0, 1.0)
}

pub fn semicircle(note: Note, time: f64) -> f64 {
    let p = phase_at(note, time);
    let sign = if p < 0.5 { 1.0 } else { -1.0 };
    let x = 2.0 * (p * 2.0).fract() - 1.0;
    sign * (1.0 - x * x).max(0.0).sqrt()
}

pub fn parabolic(note: Note, time: f64) -> f64 {
    let p = phase_at(note, time);
    let sign = if p < 0.5 { 1.0 } else { -1.0 };
    let q = (p * 2.0).fract();
    sign * 4.0 * q * (1.0 - q)
}

pub fn sinc_pulse(note: Note, time: f64, lobes: i32) -> f64 {
    let l = lobes.clamp(1, 64) as f64;
    let x = (phase_at(note, time) - 0.5) * 2.0 * l;
    if x.abs() < 1e-12 { 1.0 } else { (PI * x).sin() / (PI * x) }
}

pub fn cauchy_pulse(note: Note, time: f64, width: i32) -> f64 {
    let gamma = width.clamp(1, 100) as f64 / 100.0 * 0.25;
    let x = (phase_at(note, time) - 0.5) / gamma;
    1.0 / (1.0 + x * x)
}

pub fn exponential_saw(note: Note, time: f64, curvature: i32) -> f64 {
    let p = phase_at(note, time);
    let k = curvature.clamp(-200, 200) as f64 / 10.0;
    if k.abs() < 1e-9 {
        2.0 * p - 1.0
    } else {
        2.0 * ((k * p).exp_m1() / k.exp_m1()) - 1.0
    }
}

pub fn doublet(note: Note, time: f64) -> f64 {
    let x = (phase_at(note, time) - 0.5) / 0.05;
    -x * (0.5 - 0.5 * x * x).exp()
}

pub fn wavefolded_sine(note: Note, time: f64, fold: i32) -> f64 {
    let g = 1.0 + fold.clamp(0, 200) as f64 / 10.0;
    (g * sine(note, time)).sin()
}

pub fn soft_clipped_sine(note: Note, time: f64, drive: i32) -> f64 {
    let g = 1.0 + drive.clamp(0, 200) as f64 / 10.0;
    (g * sine(note, time)).tanh() / g.tanh()
}

pub fn self_fm_sine(note: Note, time: f64, feedback: i32) -> f64 {
    let beta = feedback.clamp(0, 100) as f64 / 10.0;
    let theta = TAU * note.frequency_hz() * time;
    (theta + beta * theta.sin()).sin()
}

pub fn chebyshev(note: Note, time: f64, harmonic: u8) -> f64 {
    let x = sine(note, time);
    match harmonic {
        0 => 1.0,
        1 => x,
        n => {
            let (mut prev, mut cur) = (1.0, x);
            for _ in 2..=n {
                let next = 2.0 * x * cur - prev;
                prev = cur;
                cur = next;
            }
            cur
        }
    }
}

pub fn weierstrass(note: Note, time: f64, terms: u8) -> f64 {
    let p = phase_at(note, time);
    let (mut sum, mut norm, mut amp, mut freq) = (0.0, 0.0, 1.0, 1.0);
    for _ in 0..terms.clamp(1, 10) {
        sum += amp * (TAU * freq * p).cos();
        norm += amp;
        amp *= 0.5;
        freq *= 3.0;
    }
    sum / norm
}

pub fn takagi(note: Note, time: f64, terms: u8) -> f64 {
    let mut x = phase_at(note, time);
    let (mut sum, mut scale) = (0.0, 1.0);
    for _ in 0..terms.clamp(1, 16) {
        sum += (x - x.round()).abs() * scale;
        x *= 2.0;
        scale *= 0.5;
    }
    sum * 1.5
}

pub fn cantor(note: Note, time: f64, depth: u8) -> f64 {
    let mut x = phase_at(note, time);
    let (mut y, mut scale) = (0.0, 0.5);
    for _ in 0..depth.clamp(1, 20) {
        x *= 3.0;
        if x >= 2.0 {
            y += scale;
            x -= 2.0;
        } else if x >= 1.0 {
            y += scale;
            break;
        }
        scale *= 0.5;
    }
    2.0 * y - 1.0
}

pub fn thue_morse(note: Note, time: f64, bits: u8) -> f64 {
    let b = bits.clamp(1, 16) as u32;
    let slots = 1u32 << b;
    let n = ((phase_at(note, time) * slots as f64) as u32).min(slots - 1);
    if n.count_ones() % 2 == 0 { 1.0 } else { -1.0 }
}
