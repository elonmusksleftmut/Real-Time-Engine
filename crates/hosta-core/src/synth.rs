//! The prototype's seven synthetic test scenes (used for regression tests and
//! for checking the classifier), plus a few extra stress signals.

use crate::fft::Radix2Fft;
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene {
    CleanMix = 0,
    OddClipper = 1,
    TubeSubImd = 2,
    DryKick = 3,
    AllPassKick = 4,
    ReversedKick = 5,
    LinearPhaseKick = 6,
}

impl Scene {
    pub const ALL: [Scene; 7] = [
        Scene::CleanMix,
        Scene::OddClipper,
        Scene::TubeSubImd,
        Scene::DryKick,
        Scene::AllPassKick,
        Scene::ReversedKick,
        Scene::LinearPhaseKick,
    ];
}

/// Mulberry32 PRNG, bit-exact with the prototype.
pub struct Mulberry32(u32);

impl Mulberry32 {
    pub fn new(seed: u32) -> Self {
        Self(seed)
    }
    pub fn next_f64(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6D2B79F5);
        let a = self.0;
        let mut t = (a ^ (a >> 15)).wrapping_mul(1 | a);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        (t ^ (t >> 14)) as f64 / 4294967296.0
    }
    pub fn gauss(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-12);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
    }
}

/// One dry 125 BPM (0.48 s) causal kick: 62.5 Hz sub + 250 Hz multi-harmonic click, onset at 80 ms.
pub fn dry_kick_beat(fs: f64, beat_sec: f64) -> Vec<f64> {
    let n = (fs * beat_sec).round() as usize;
    let t_onset = 0.08;
    (0..n)
        .map(|i| {
            let t = i as f64 / fs;
            if t < t_onset {
                return 0.0;
            }
            let dt = t - t_onset;
            let attack = 1.0 - (-dt / 0.0012).exp();
            let env_sub = attack * (-dt / 0.085).exp();
            let env_saw = attack * (-dt / 0.055).exp();
            let sub = 0.55 * (2.0 * PI * 62.5 * dt).sin();
            let saw: f64 = (1..=6)
                .map(|h| (2.0 * PI * (250.0 * h as f64) * dt).sin() / (h as f64).powf(0.85))
                .sum();
            0.52 * (env_sub * sub + 0.65 * env_saw * saw)
        })
        .collect()
}

/// 10-stage 2nd-order all-pass cascade (|H| = 1), circularly wrapped over one beat.
pub fn all_pass10(beat: &[f64], fs: f64, f0: f64, r_at_8k: f64) -> Vec<f64> {
    let n = beat.len();
    let r = r_at_8k.powf(8000.0 / fs);
    let w0 = 2.0 * PI * f0 / fs;
    let a1 = -2.0 * r * w0.cos();
    let a2 = r * r;
    let mut x: Vec<f64> = beat.iter().chain(beat.iter()).copied().collect();
    let mut y = vec![0.0; 2 * n];
    for _ in 0..10 {
        let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
        for i in 0..2 * n {
            let x0 = x[i];
            let y0 = a2 * x0 + a1 * x1 + x2 - a1 * y1 - a2 * y2;
            y[i] = y0;
            x2 = x1;
            x1 = x0;
            y2 = y1;
            y1 = y0;
        }
        std::mem::swap(&mut x, &mut y);
    }
    x[n..].to_vec()
}

/// Zero-phase Blackman-windowed sinc lowpass (pre-rings before the onset).
pub fn linear_phase_lowpass(beat: &[f64], fs: f64, fc: f64) -> Vec<f64> {
    let n = beat.len();
    let half_m = (0.036 * fs).round() as usize;
    let l = 2 * half_m + 1;
    let mut h: Vec<f64> = (0..l)
        .map(|i| {
            let k = i as f64 - half_m as f64;
            let sinc = if k == 0.0 {
                2.0 * fc / fs
            } else {
                (2.0 * PI * fc * k / fs).sin() / (PI * k)
            };
            let w = 0.42 - 0.5 * (2.0 * PI * i as f64 / (l - 1) as f64).cos()
                + 0.08 * (4.0 * PI * i as f64 / (l - 1) as f64).cos();
            sinc * w
        })
        .collect();
    let s: f64 = h.iter().sum();
    h.iter_mut().for_each(|v| *v /= s);

    let n_fft = (n + l).next_power_of_two();
    let fft = Radix2Fft::new(n_fft);
    let (mut xr, mut xi) = (vec![0.0; n_fft], vec![0.0; n_fft]);
    let (mut hr, mut hi) = (vec![0.0; n_fft], vec![0.0; n_fft]);
    xr[..n].copy_from_slice(beat);
    for (i, &v) in h.iter().enumerate() {
        let idx = (i as isize - half_m as isize).rem_euclid(n_fft as isize) as usize;
        hr[idx] = v;
    }
    fft.transform(&mut xr, &mut xi, false);
    fft.transform(&mut hr, &mut hi, false);
    for i in 0..n_fft {
        let re = xr[i] * hr[i] - xi[i] * hi[i];
        let im = xr[i] * hi[i] + xi[i] * hr[i];
        xr[i] = re;
        xi[i] = im;
    }
    fft.transform(&mut xr, &mut xi, true);
    let mut out = xr[..n].to_vec();
    for i in n..n_fft {
        out[i % n] += xr[i];
    }
    out
}

/// Render one loop of a scene at native rate `fs` (same seeds as the prototype).
pub fn render_scene(scene: Scene, fs: f64) -> Vec<f32> {
    let id = scene as u32;
    let mut rng = Mulberry32::new(202611 + id * 7919);
    if id <= 2 {
        let n_segs = 46;
        let seg = (0.13 * fs).round() as usize;
        let fade = ((0.006 * fs).round() as usize).max(16);
        let mut ph = vec![[0.0f64; 3]; n_segs];
        for p in ph.iter_mut() {
            for v in p.iter_mut() {
                *v = rng.next_f64() * 2.0 * PI;
            }
        }
        let eval = |t: f64, p: &[f64; 3]| -> f64 {
            match scene {
                Scene::CleanMix => {
                    0.36 * (2.0 * PI * 62.5 * t + p[0]).cos()
                        + 0.36 * (2.0 * PI * 281.25 * t + p[1]).cos()
                        + 0.28 * (2.0 * PI * 343.75 * t + p[2]).cos()
                }
                Scene::OddClipper => {
                    let raw = 0.68 * (2.0 * PI * 93.75 * t + p[0]).cos()
                        + 0.58 * (2.0 * PI * 156.25 * t + p[1]).cos();
                    0.62 * (3.2 * raw).tanh()
                }
                _ => {
                    let raw = 0.62 * (2.0 * PI * 62.5 * t + p[0]).cos()
                        + 0.52 * (2.0 * PI * 281.25 * t + p[1]).cos();
                    let dc = 0.45 * 0.5 * (0.62 * 0.62 + 0.52 * 0.52);
                    0.55 * (raw + 0.45 * raw * raw - dc)
                }
            }
        };
        let noise_std = if scene == Scene::CleanMix {
            0.075
        } else {
            0.045
        };
        let mut out = vec![0.0f32; n_segs * seg];
        for s in 0..n_segs {
            let prev = (s + n_segs - 1) % n_segs;
            for j in 0..seg {
                let idx = s * seg + j;
                let t = idx as f64 / fs;
                let mut v = eval(t, &ph[s]);
                if j < fade {
                    let w = 0.5 - 0.5 * (PI * j as f64 / fade as f64).cos();
                    v = eval(t, &ph[prev]) * (1.0 - w) + v * w;
                }
                out[idx] = (v + noise_std * rng.gauss()) as f32;
            }
        }
        return out;
    }

    let dry = dry_kick_beat(fs, 0.48);
    let beat = match scene {
        Scene::DryKick => dry,
        Scene::AllPassKick => all_pass10(&dry, fs, 260.0, 0.90),
        Scene::ReversedKick => dry.iter().rev().copied().collect(),
        _ => linear_phase_lowpass(&dry, fs, 420.0),
    };
    let mut out = Vec::with_capacity(beat.len() * 4);
    for _ in 0..4 {
        for &v in &beat {
            out.push((v + 0.012 * rng.gauss()) as f32);
        }
    }
    out
}

// ---- extra stress signals (not in the prototype) -----------------------------

pub fn white_noise(fs: f64, secs: f64, std: f64, seed: u32) -> Vec<f32> {
    let mut rng = Mulberry32::new(seed);
    (0..(fs * secs) as usize)
        .map(|_| (std * rng.gauss()) as f32)
        .collect()
}

pub fn sine(fs: f64, secs: f64, f: f64, amp: f64) -> Vec<f32> {
    (0..(fs * secs) as usize)
        .map(|i| (amp * (2.0 * PI * f * i as f64 / fs).sin()) as f32)
        .collect()
}

/// Band-limited sawtooth (harmonics below 0.45·fs): a *clean* but fully
/// phase-locked harmonic tone. Bicoherence flags it as coupled by design.
pub fn saw(fs: f64, secs: f64, f: f64, amp: f64) -> Vec<f32> {
    let hmax = ((0.45 * fs) / f) as usize;
    (0..(fs * secs) as usize)
        .map(|i| {
            let t = i as f64 / fs;
            let v: f64 = (1..=hmax)
                .map(|h| (2.0 * PI * f * h as f64 * t).sin() / h as f64)
                .sum();
            (amp * v * 2.0 / PI) as f32
        })
        .collect()
}
