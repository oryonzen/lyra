#![allow(unused)]

use std::f64::consts::{PI, TAU};
use crate::core::c_pitch::Note;

#[derive(Debug, Clone, Copy)]
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

pub fn waveform(wave: Waveform, note: Note, time: f64, volume: f64) -> f64 {
    let hz = note.frequency_hz();
    let p = (hz * time).rem_euclid(1.0);
    let theta = TAU * hz * time;

    volume * match wave {
        Waveform::Sine => theta.sin(),
        Waveform::Sawtooth => 2.0 * p - 1.0,
        Waveform::Square => if p < 0.5 { 1.0 } else { -1.0 },
        Waveform::Triangle => 1.0 - 4.0 * (p - 0.5).abs(),
        Waveform::Pulse(width) => {
            let duty = width.clamp(0, 100) as f64 / 100.0;
            if p < duty { 1.0 } else { -1.0 }
        }
        Waveform::ReverseSawtooth => 1.0 - 2.0 * p,
        Waveform::HalfRectSine => theta.sin().max(0.0),
        Waveform::FullRectSine => theta.sin().abs(),
        Waveform::ImpulseTrain => if p < 0.03 { 1.0 } else { 0.0 },
        Waveform::Staircase(steps) => {
            let n = steps.clamp(2, 256) as f64;
            let level = (p * n).floor().min(n - 1.0);
            2.0 * level / (n - 1.0) - 1.0
        }
        Waveform::SineCubed => theta.sin().powi(3),
        Waveform::GaussianPulse(width) => {
            let sigma = width.clamp(1, 100) as f64 / 100.0 * 0.25;
            let x = (p - 0.5) / sigma;
            (-0.5 * x * x).exp()
        }
        Waveform::Trapezoid(flat_top) => {
            let flat = flat_top.clamp(0, 99) as f64 / 100.0;
            let tri = 1.0 - 4.0 * (p - 0.5).abs();
            (tri / (1.0 - flat)).clamp(-1.0, 1.0)
        }
        Waveform::Semicircle => {
            let sign = if p < 0.5 { 1.0 } else { -1.0 };
            let x = 2.0 * (p * 2.0).fract() - 1.0;
            sign * (1.0 - x * x).max(0.0).sqrt()
        }
        Waveform::Parabolic => {
            let sign = if p < 0.5 { 1.0 } else { -1.0 };
            let q = (p * 2.0).fract();
            sign * 4.0 * q * (1.0 - q)
        }
        Waveform::SincPulse(lobes) => {
            let l = lobes.clamp(1, 64) as f64;
            let x = (p - 0.5) * 2.0 * l;
            if x.abs() < 1e-12 { 1.0 } else { (PI * x).sin() / (PI * x) }
        }
        Waveform::CauchyPulse(width) => {
            let gamma = width.clamp(1, 100) as f64 / 100.0 * 0.25;
            let x = (p - 0.5) / gamma;
            1.0 / (1.0 + x * x)
        }
        Waveform::ExponentialSaw(curvature) => {
            let k = curvature.clamp(-200, 200) as f64 / 10.0;
            if k.abs() < 1e-9 {
                2.0 * p - 1.0
            } else {
                2.0 * ((k * p).exp_m1() / k.exp_m1()) - 1.0
            }
        }
        Waveform::Doublet => {
            let x = (p - 0.5) / 0.05;
            -x * (0.5 - 0.5 * x * x).exp()
        }
        Waveform::WavefoldedSine(fold) => {
            let g = 1.0 + fold.clamp(0, 200) as f64 / 10.0;
            (g * theta.sin()).sin()
        }
        Waveform::SoftClippedSine(drive) => {
            let g = 1.0 + drive.clamp(0, 200) as f64 / 10.0;
            (g * theta.sin()).tanh() / g.tanh()
        }
        Waveform::SelfFMSine(feedback) => {
            let beta = feedback.clamp(0, 100) as f64 / 10.0;
            (theta + beta * theta.sin()).sin()
        }
        Waveform::Chebyshev(harmonic) => {
            let x = theta.sin();
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
        Waveform::Weierstrass(terms) => {
            let (mut sum, mut norm, mut amp, mut freq) = (0.0, 0.0, 1.0, 1.0);
            for _ in 0..terms.clamp(1, 10) {
                sum += amp * (TAU * freq * p).cos();
                norm += amp;
                amp *= 0.5;
                freq *= 3.0;
            }
            sum / norm
        }
        Waveform::Takagi(terms) => {
            let mut x = p;
            let (mut sum, mut scale) = (0.0, 1.0);
            for _ in 0..terms.clamp(1, 16) {
                sum += (x - x.round()).abs() * scale;
                x *= 2.0;
                scale *= 0.5;
            }
            sum * 1.5
        }
        Waveform::Cantor(depth) => {
            let mut x = p;
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
        Waveform::ThueMorse(bits) => {
            let b = bits.clamp(1, 16) as u32;
            let slots = 1u32 << b;
            let n = ((p * slots as f64) as u32).min(slots - 1);
            if n.count_ones() % 2 == 0 { 1.0 } else { -1.0 }
        }
    }
}
