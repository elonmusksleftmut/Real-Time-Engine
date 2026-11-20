//! Rolling-EMA higher-order spectral engine (3rd-order bicoherence, 4th-order
//! tricoherence) plus the causal time-arrow estimators.
//!
//! This is a line-by-line port of `RollingSpectralEngine` from the browser
//! prototype (`time.py`). Nothing allocates after construction, so it is safe on
//! an audio thread. With [`EngineOptions::legacy`] it reproduces the prototype's
//! numbers; by default it adds two accuracy fixes:
//!
//! * **No per-frame mean subtraction.** Subtracting a frame's mean before
//!   windowing injects `mean · w[n]` into bins 0–1, and that mean is a function
//!   of every tone's phase. A single sine then looks phase-coupled to itself
//!   (quartets `(f, 1, 1 → f+2)` light up at t² ≈ 0.65). DC is removed upstream
//!   by a 5 Hz blocker instead ([`crate::analyzer::Analyzer`]).
//! * **Per-bin coupling floor.** A triad/quartet is only scored when every bin
//!   it touches holds at least `floor` × the loudest bin's rolling power. Hann
//!   sidelobes of a pure tone are themselves phase-locked to it, so without a
//!   floor a digital test tone reads as "ODD CLIP".
//!
//! The readout also carries a noise-bias-corrected Im[B] RMS; the verdict comes
//! from [`crate::classify`].

use crate::classify::{self, Verdict};
use crate::fft::Radix2Fft;

pub const N_FFT: usize = 256;
pub const HOP: usize = 128;
/// Decimated samples per time-arrow update (the prototype's worklet chunk size).
pub const CHUNK: usize = 64;
pub const BI_MAX_SUM: usize = 64;
pub const TRI_MAX_SUM: usize = 36;
pub const BI_W: usize = 64;
pub const BI_H: usize = 32;
pub const TRI_W: usize = 36;
pub const TRI_H: usize = 18;
pub const RTA_BINS: usize = N_FFT / 2 + 1;
/// Oscilloscope ring (~0.5 s at 8 kHz, so a 125 BPM beat is always in view). Power of two.
pub const SCOPE_LEN: usize = 4096;
pub const PEAK_RING_LEN: usize = 80;
/// Family-wise false-alarm rate for the Bonferroni significance threshold.
pub const ALPHA: f64 = 0.05;

/// Engine behaviour switches.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EngineOptions {
    /// Reproduce the browser prototype exactly (per-frame mean removal, no floor).
    pub legacy: bool,
    /// Coupling floor in dB relative to the loudest bin (ignored in legacy mode).
    pub floor_db: f64,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            legacy: false,
            floor_db: -70.0,
        }
    }
}

impl EngineOptions {
    pub const LEGACY: Self = Self {
        legacy: true,
        floor_db: -300.0,
    };
}

const MAX_LAG: usize = (N_FFT - 1) / HOP;

/// Static index tables for the principal bispectral triangle and the positive
/// tri-spectral sum octant. Deterministic, so the GUI can build its own copy.
pub struct Layout {
    pub bi_i1: Vec<u16>,
    pub bi_i2: Vec<u16>,
    pub bi_i3: Vec<u16>,
    /// `BI_W * BI_H` grid cell -> flat triad index (or -1 outside the triangle).
    pub bi_cell_to_flat: Vec<i32>,
    pub tri_i1: Vec<u16>,
    pub tri_i2: Vec<u16>,
    pub tri_i3: Vec<u16>,
    pub tri_i4: Vec<u16>,
    pub tri_valid_proj: Vec<bool>,
}

impl Layout {
    pub fn new() -> Self {
        let mut l = Layout {
            bi_i1: vec![],
            bi_i2: vec![],
            bi_i3: vec![],
            bi_cell_to_flat: vec![-1; BI_W * BI_H],
            tri_i1: vec![],
            tri_i2: vec![],
            tri_i3: vec![],
            tri_i4: vec![],
            tri_valid_proj: vec![false; TRI_W * TRI_H],
        };
        for i2 in 1..=BI_MAX_SUM / 2 {
            for i1 in i2..=BI_MAX_SUM - i2 {
                l.bi_cell_to_flat[(i2 - 1) * BI_W + (i1 - 1)] = l.bi_i1.len() as i32;
                l.bi_i1.push(i1 as u16);
                l.bi_i2.push(i2 as u16);
                l.bi_i3.push((i1 + i2) as u16);
            }
        }
        for i3 in 1..=TRI_MAX_SUM / 3 {
            for i2 in i3..=(TRI_MAX_SUM - i3) / 2 {
                for i1 in i2..=TRI_MAX_SUM - i3 - i2 {
                    l.tri_i1.push(i1 as u16);
                    l.tri_i2.push(i2 as u16);
                    l.tri_i3.push(i3 as u16);
                    l.tri_i4.push((i1 + i2 + i3) as u16);
                    l.tri_valid_proj[(i2 - 1) * TRI_W + (i1 - 1)] = true;
                }
            }
        }
        l
    }

    pub fn n_triads(&self) -> usize {
        self.bi_i1.len()
    }

    pub fn n_quartets(&self) -> usize {
        self.tri_i1.len()
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Triad {
    pub f1: f64,
    pub f2: f64,
    pub f3: f64,
    pub b2: f64,
    pub psi_deg: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quartet {
    pub f1: f64,
    pub f2: f64,
    pub f3: f64,
    pub f4: f64,
    pub t2: f64,
    pub psi_deg: f64,
}

/// Everything the HUD, the classifier and the control outputs need.
#[derive(Clone, Copy, Debug, Default)]
pub struct Readout {
    pub k_eff3: f64,
    pub k_eff4: f64,
    pub crit_bi: f64,
    pub crit_tri: f64,
    pub bi_hits: u32,
    pub tri_hits: u32,
    pub b2_max: f64,
    pub t2_max: f64,
    pub top_triad: Option<Triad>,
    pub top_quartet: Option<Quartet>,
    /// Lag-1 increment skewness (> 0: fast rise / slow decay, i.e. forward time).
    pub phi_rev: f64,
    /// Jensen-Shannon divergence between forward and time-reversed D=3 ordinal transitions (nats).
    pub d_js: f64,
    /// Prototype metric: sqrt(mean over all triads of b² sin²ψ). Has a noise floor of ~sqrt(1/2K).
    pub im_rms_raw: f64,
    /// Same, with the expected noise contribution 1/(2·K_eff) subtracted.
    pub im_rms: f64,
    pub im_polarity: f64,
    /// Mean |sin ψ| over Bonferroni-significant triads (0 = real biphase, 1 = quadrature).
    pub sig_im_mean: f64,
    pub crest: f64,
    pub h_spec: f64,
    pub active_hops: u64,
    pub gated: bool,
    pub verdict: Verdict,
    /// Verdict from the prototype's original rule set, kept for comparison.
    pub verdict_legacy: Verdict,
    pub hop_us: f64,
}

pub struct SpectralEngine {
    pub layout: Layout,
    fft: Radix2Fft,
    fft_re: Vec<f64>,
    fft_im: Vec<f64>,
    win: Vec<f64>,
    win_rho: [f64; MAX_LAG + 1],

    pub opts: EngineOptions,
    floor_lin: f64,
    bin_live: [bool; RTA_BINS],
    pub fs_a: f64,
    pub df: f64,
    pub tau_sec: f64,
    pub gate_db: f64,
    gate_rms: f64,
    lam_hop: f64,
    ovlp3: f64,
    ovlp4: f64,

    nbi_re: Vec<f64>,
    nbi_im: Vec<f64>,
    du_bi: Vec<f64>,
    dv_bi: Vec<f64>,
    pub b2: Vec<f32>,
    pub biphase: Vec<f32>,

    ntri_re: Vec<f64>,
    ntri_im: Vec<f64>,
    du_tri: Vec<f64>,
    dv_tri: Vec<f64>,
    pub t2: Vec<f32>,
    pub triphase: Vec<f32>,

    pub tri_proj_val: Vec<f32>,
    pub tri_proj_phase: Vec<f32>,
    pub tri_proj_best_i3: Vec<u16>,
    pub tri_proj_sig: Vec<bool>,

    pub rta_power: [f64; RTA_BINS],
    pub scope_ring: [f32; SCOPE_LEN],
    pub scope_write: usize,
    peak_ring: [f32; PEAK_RING_LEN],
    peak_write: usize,

    stft_ring: [f64; N_FFT],
    hop_fill: usize,
    total_samples: u64,
    pub active_hops: u64,
    pub is_gated: bool,

    skew_tail: f64,
    skew_primed: bool,
    m2_dx: f64,
    m3_dx: f64,
    ms_signal: f64,
    ord_tail: [f64; 3],
    ord_primed: bool,
    ord_buf: [f64; CHUNK + 3],
    p_ord: [f64; 64],
    rng_state: u32,
    h_spec_ema: f64,
    last_hop_us: f64,

    pub latest: Readout,
}

/// Reverse-time map of a D=3 ordinal code (bit-exact port of `revOrd3Code`).
const fn rev_ord3(c: usize) -> usize {
    let b2 = (c >> 2) & 1;
    let b1 = (c >> 1) & 1;
    let b0 = c & 1;
    ((1 - b1) << 2) | ((1 - b2) << 1) | (1 - b0)
}

/// Transition (code1 -> code2) under time reversal becomes (rev code2 -> rev code1).
pub const REV_LUT: [u8; 64] = {
    let mut lut = [0u8; 64];
    let mut p = 0;
    while p < 64 {
        let c1 = (p >> 3) & 7;
        let c2 = p & 7;
        lut[p] = ((rev_ord3(c2) << 3) | rev_ord3(c1)) as u8;
        p += 1;
    }
    lut
};

impl SpectralEngine {
    pub fn new(fs_a: f64, tau_sec: f64, gate_db: f64) -> Self {
        let layout = Layout::new();
        let nt = layout.n_triads();
        let nq = layout.n_quartets();

        let win: Vec<f64> = (0..N_FFT)
            .map(|n| 0.5 * (1.0 - (2.0 * std::f64::consts::PI * n as f64 / N_FFT as f64).cos()))
            .collect();
        let w_energy: f64 = win.iter().map(|w| w * w).sum();
        let mut win_rho = [0.0; MAX_LAG + 1];
        win_rho[0] = 1.0;
        for (m, rho) in win_rho.iter_mut().enumerate().skip(1) {
            let shift = m * HOP;
            *rho = (0..N_FFT - shift)
                .map(|n| win[n] * win[n + shift])
                .sum::<f64>()
                / w_energy;
        }

        let mut e = Self {
            layout,
            fft: Radix2Fft::new(N_FFT),
            fft_re: vec![0.0; N_FFT],
            fft_im: vec![0.0; N_FFT],
            win,
            win_rho,
            opts: EngineOptions::default(),
            floor_lin: 1e-7,
            bin_live: [true; RTA_BINS],
            fs_a: 8000.0,
            df: 8000.0 / N_FFT as f64,
            tau_sec: 0.6,
            gate_db: -55.0,
            gate_rms: 0.0,
            lam_hop: 0.0,
            ovlp3: 1.0,
            ovlp4: 1.0,
            nbi_re: vec![0.0; nt],
            nbi_im: vec![0.0; nt],
            du_bi: vec![0.0; nt],
            dv_bi: vec![0.0; nt],
            b2: vec![0.0; nt],
            biphase: vec![0.0; nt],
            ntri_re: vec![0.0; nq],
            ntri_im: vec![0.0; nq],
            du_tri: vec![0.0; nq],
            dv_tri: vec![0.0; nq],
            t2: vec![0.0; nq],
            triphase: vec![0.0; nq],
            tri_proj_val: vec![0.0; TRI_W * TRI_H],
            tri_proj_phase: vec![0.0; TRI_W * TRI_H],
            tri_proj_best_i3: vec![0; TRI_W * TRI_H],
            tri_proj_sig: vec![false; TRI_W * TRI_H],
            rta_power: [0.0; RTA_BINS],
            scope_ring: [0.0; SCOPE_LEN],
            scope_write: 0,
            peak_ring: [0.0; PEAK_RING_LEN],
            peak_write: 0,
            stft_ring: [0.0; N_FFT],
            hop_fill: 0,
            total_samples: 0,
            active_hops: 0,
            is_gated: false,
            skew_tail: 0.0,
            skew_primed: false,
            m2_dx: 0.0,
            m3_dx: 0.0,
            ms_signal: 0.0,
            ord_tail: [0.0; 3],
            ord_primed: false,
            ord_buf: [0.0; CHUNK + 3],
            p_ord: [0.0; 64],
            rng_state: 0x13579bdf,
            h_spec_ema: 0.0,
            last_hop_us: 0.0,
            latest: Readout::default(),
        };
        e.configure(fs_a, tau_sec, gate_db);
        e.set_options(EngineOptions::default());
        e.reset();
        e
    }

    pub fn set_options(&mut self, opts: EngineOptions) {
        self.opts = opts;
        self.floor_lin = if opts.legacy {
            0.0
        } else {
            10f64.powf(opts.floor_db / 10.0)
        };
    }

    pub fn configure(&mut self, fs_a: f64, tau_sec: f64, gate_db: f64) {
        self.fs_a = fs_a.max(500.0);
        self.df = self.fs_a / N_FFT as f64;
        self.tau_sec = tau_sec.clamp(0.2, 2.5);
        self.gate_db = gate_db;
        self.gate_rms = 10f64.powf(gate_db / 20.0);
        let dt_hop = HOP as f64 / self.fs_a;
        self.lam_hop = 1.0 - (-dt_hop / self.tau_sec).exp();
        let (mut c3, mut c4) = (0.0, 0.0);
        for m in 1..self.win_rho.len() {
            let rho = self.win_rho[m];
            let dec_m = (1.0 - self.lam_hop).powi(m as i32);
            c3 += dec_m * rho * rho * rho;
            c4 += dec_m * rho * rho * rho * rho;
        }
        self.ovlp3 = 1.0 + 2.0 * c3;
        self.ovlp4 = 1.0 + 2.0 * c4;
    }

    pub fn reset(&mut self) {
        for v in [
            &mut self.nbi_re,
            &mut self.nbi_im,
            &mut self.du_bi,
            &mut self.dv_bi,
        ] {
            v.fill(0.0);
        }
        for v in [
            &mut self.ntri_re,
            &mut self.ntri_im,
            &mut self.du_tri,
            &mut self.dv_tri,
        ] {
            v.fill(0.0);
        }
        for v in [
            &mut self.b2,
            &mut self.biphase,
            &mut self.t2,
            &mut self.triphase,
        ] {
            v.fill(0.0);
        }
        self.tri_proj_val.fill(0.0);
        self.tri_proj_phase.fill(0.0);
        self.tri_proj_best_i3.fill(0);
        self.tri_proj_sig.fill(false);
        self.rta_power = [1e-12; RTA_BINS];
        self.bin_live = [true; RTA_BINS];
        self.scope_ring = [0.0; SCOPE_LEN];
        self.scope_write = 0;
        self.peak_ring = [0.0; PEAK_RING_LEN];
        self.peak_write = 0;
        self.stft_ring = [0.0; N_FFT];
        self.hop_fill = 0;
        self.total_samples = 0;
        self.active_hops = 0;
        self.is_gated = false;
        self.skew_tail = 0.0;
        self.skew_primed = false;
        self.m2_dx = 0.0;
        self.m3_dx = 0.0;
        self.ms_signal = 0.0;
        self.ord_tail = [0.0; 3];
        self.ord_primed = false;
        self.p_ord = [1.0 / 64.0; 64];
        self.h_spec_ema = 0.0;
        self.last_hop_us = 0.0;
        self.latest = Readout::default();
    }

    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }

    /// Xorshift32 symmetric triangular dither in [-1, 1] (bit-exact with the prototype).
    #[inline]
    fn sym_dither(&mut self) -> f64 {
        let mut s = self.rng_state;
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        let u1 = s as f64 * 2.3283064365386963e-10;
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        let u2 = s as f64 * 2.3283064365386963e-10;
        self.rng_state = s;
        u1 - u2
    }

    fn k_eff(&self, order: u32) -> f64 {
        let n = self.active_hops;
        if n <= 1 {
            return n as f64;
        }
        let lam = self.lam_hop;
        let rn = (1.0 - lam).powf(n as f64);
        let num = (1.0 - rn) * (1.0 - rn);
        let den = (1.0 - rn * rn).max(1e-30);
        let k_ema = ((2.0 - lam) / lam) * (num / den);
        let ovlp = if order == 3 { self.ovlp3 } else { self.ovlp4 };
        (k_ema / ovlp).max(1.0)
    }

    /// Causal O(B) update of the increment skewness, ordinal histogram and crest factor.
    fn update_time_arrow_chunk(&mut self, chunk: &[f32]) {
        let b = chunk.len();
        if b == 0 {
            return;
        }
        assert!(b <= CHUNK);
        let mut sum_sq = 0.0;
        let mut blk_peak = 0.0f32;
        for &v in chunk {
            self.scope_ring[self.scope_write] = v;
            self.scope_write = (self.scope_write + 1) & (SCOPE_LEN - 1);
            let v64 = v as f64;
            sum_sq += v64 * v64;
            blk_peak = blk_peak.max(v.abs());
        }
        let rms = (sum_sq / b as f64).sqrt();
        if rms < self.gate_rms || b < 4 {
            self.is_gated = true;
            return;
        }
        self.is_gated = false;

        self.peak_ring[self.peak_write] = blk_peak;
        self.peak_write = (self.peak_write + 1) % PEAK_RING_LEN;

        let lam_b = 1.0 - (-(b as f64 / self.fs_a) / self.tau_sec).exp();
        let dec_b = 1.0 - lam_b;
        self.ms_signal = dec_b * self.ms_signal + lam_b * (sum_sq / b as f64);

        // 1. Lag-1 increment skewness with a one-sample carry across chunk seams.
        let (mut s2, mut s3, mut cnt) = (0.0, 0.0, 0usize);
        let mut prev = self.skew_tail;
        let mut start = 0;
        if !self.skew_primed {
            prev = chunk[0] as f64;
            start = 1;
            self.skew_primed = true;
        }
        for &cur in &chunk[start..] {
            let dx = cur as f64 - prev;
            let dx2 = dx * dx;
            s2 += dx2;
            s3 += dx2 * dx;
            cnt += 1;
            prev = cur as f64;
        }
        self.skew_tail = chunk[b - 1] as f64;
        if cnt > 0 {
            self.m2_dx = dec_b * self.m2_dx + lam_b * (s2 / cnt as f64);
            self.m3_dx = dec_b * self.m3_dx + lam_b * (s3 / cnt as f64);
        }

        // 2. D=3 Bandt-Pompe ordinal transition histogram (3-sample carry + sub-LSB dither).
        let dither_amp = 1e-6 * rms;
        let mut hist = [0.0f64; 64];
        let mut off = 0;
        if self.ord_primed {
            self.ord_buf[..3].copy_from_slice(&self.ord_tail);
            off = 3;
        } else {
            self.ord_primed = true;
        }
        let n_ext = off + b;
        for i in 0..b {
            self.ord_buf[off + i] = chunk[i] as f64 + dither_amp * self.sym_dither();
        }
        self.ord_tail
            .copy_from_slice(&self.ord_buf[n_ext - 3..n_ext]);

        let mut n_pairs = 0usize;
        let mut prev_code: Option<usize> = None;
        for w in self.ord_buf[..n_ext].windows(3) {
            let (x0, x1, x2) = (w[0], w[1], w[2]);
            let code = ((x0 < x1) as usize) << 2 | ((x1 < x2) as usize) << 1 | (x0 < x2) as usize;
            if let Some(pc) = prev_code {
                hist[(pc << 3) | code] += 1.0;
                n_pairs += 1;
            }
            prev_code = Some(code);
        }
        if n_pairs > 0 {
            let inv = 1.0 / n_pairs as f64;
            for (p, h) in self.p_ord.iter_mut().zip(hist.iter()) {
                *p = dec_b * *p + lam_b * (h * inv);
            }
        }
    }

    fn process_hop(&mut self) {
        let t0 = std::time::Instant::now();
        let n = N_FFT;
        let mean = if self.opts.legacy {
            self.stft_ring.iter().sum::<f64>() / n as f64
        } else {
            0.0
        };
        let mut sum_sq = 0.0;
        for i in 0..n {
            let xc = self.stft_ring[i] - mean;
            sum_sq += xc * xc;
            self.fft_re[i] = xc * self.win[i];
            self.fft_im[i] = 0.0;
        }
        let rms = (sum_sq / n as f64).sqrt();
        if rms < self.gate_rms {
            self.is_gated = true;
            self.last_hop_us = t0.elapsed().as_secs_f64() * 1e6;
            return;
        }
        self.is_gated = false;

        self.fft
            .transform(&mut self.fft_re, &mut self.fft_im, false);
        let (re, im) = (&self.fft_re, &self.fft_im);
        let lam = self.lam_hop;
        let dec = 1.0 - lam;
        self.active_hops += 1;

        // 2nd-order RTA + normalized spectral entropy.
        let inv_scale = 1.0 / (n * n) as f64;
        let mut p_sum = 0.0;
        for b in 0..RTA_BINS {
            let p = (re[b] * re[b] + im[b] * im[b]) * inv_scale;
            self.rta_power[b] = dec * self.rta_power[b] + lam * p;
            if b >= 1 && b < RTA_BINS - 1 {
                p_sum += p;
            }
        }
        if p_sum > 1e-30 {
            let inv_p = 1.0 / p_sum;
            let mut ent = 0.0;
            for b in 1..RTA_BINS - 1 {
                let pb = (re[b] * re[b] + im[b] * im[b]) * inv_scale * inv_p;
                if pb > 1e-15 {
                    ent -= pb * pb.ln();
                }
            }
            let h_norm = ent / ((RTA_BINS - 2) as f64).ln();
            self.h_spec_ema = dec * self.h_spec_ema + lam * h_norm;
        }

        // 3rd order.
        let l = &self.layout;
        for k in 0..l.bi_i1.len() {
            let (a, b, c) = (
                l.bi_i1[k] as usize,
                l.bi_i2[k] as usize,
                l.bi_i3[k] as usize,
            );
            let ur = re[a] * re[b] - im[a] * im[b];
            let ui = re[a] * im[b] + im[a] * re[b];
            let (vr, vi) = (re[c], im[c]);
            let pr = ur * vr + ui * vi;
            let pi = ui * vr - ur * vi;
            self.nbi_re[k] = dec * self.nbi_re[k] + lam * pr;
            self.nbi_im[k] = dec * self.nbi_im[k] + lam * pi;
            self.du_bi[k] = dec * self.du_bi[k] + lam * (ur * ur + ui * ui);
            self.dv_bi[k] = dec * self.dv_bi[k] + lam * (vr * vr + vi * vi);
        }

        // 4th order.
        for k in 0..l.tri_i1.len() {
            let (a, b, c, d) = (
                l.tri_i1[k] as usize,
                l.tri_i2[k] as usize,
                l.tri_i3[k] as usize,
                l.tri_i4[k] as usize,
            );
            let abr = re[a] * re[b] - im[a] * im[b];
            let abi = re[a] * im[b] + im[a] * re[b];
            let ur = abr * re[c] - abi * im[c];
            let ui = abr * im[c] + abi * re[c];
            let (vr, vi) = (re[d], im[d]);
            let pr = ur * vr + ui * vi;
            let pi = ui * vr - ur * vi;
            self.ntri_re[k] = dec * self.ntri_re[k] + lam * pr;
            self.ntri_im[k] = dec * self.ntri_im[k] + lam * pi;
            self.du_tri[k] = dec * self.du_tri[k] + lam * (ur * ur + ui * ui);
            self.dv_tri[k] = dec * self.dv_tri[k] + lam * (vr * vr + vi * vi);
        }
        self.last_hop_us = t0.elapsed().as_secs_f64() * 1e6;
    }

    /// Push one chunk of decimated audio (at most [`CHUNK`] samples).
    /// Returns the number of STFT hops that completed inside it.
    pub fn push_decimated(&mut self, chunk: &[f32]) -> usize {
        self.update_time_arrow_chunk(chunk);
        let mut hops = 0;
        let mut pos = 0;
        while pos < chunk.len() {
            let take = (HOP - self.hop_fill).min(chunk.len() - pos);
            self.stft_ring.copy_within(take.., 0);
            for i in 0..take {
                self.stft_ring[N_FFT - take + i] = chunk[pos + i] as f64;
            }
            self.hop_fill += take;
            self.total_samples += take as u64;
            pos += take;
            if self.hop_fill == HOP {
                self.hop_fill = 0;
                if self.total_samples >= N_FFT as u64 {
                    self.process_hop();
                    hops += 1;
                }
            }
        }
        hops
    }

    /// Recompute all derived statistics. Cheap enough (~2.4k atan2) to run every hop.
    /// Whether bin `b` is above the coupling floor (as of the last readout).
    pub fn bin_live(&self, b: usize) -> bool {
        self.bin_live[b]
    }

    pub fn compute_readout(&mut self) -> Readout {
        let nt = self.layout.n_triads();
        let nq = self.layout.n_quartets();
        let k3 = self.k_eff(3);
        let k4 = self.k_eff(4);
        let crit_bi = if k3 > 1.05 {
            1.0 - (ALPHA / nt as f64).powf(1.0 / (k3 - 1.0))
        } else {
            1.0
        };
        let crit_tri = if k4 > 1.05 {
            1.0 - (ALPHA / nq as f64).powf(1.0 / (k4 - 1.0))
        } else {
            1.0
        };

        // Per-bin liveness against the loudest bin anywhere in the spectrum.
        let peak = self.rta_power[1..].iter().copied().fold(0.0, f64::max);
        let thr = self.floor_lin * peak;
        for (live, &p) in self.bin_live.iter_mut().zip(self.rta_power.iter()) {
            *live = p >= thr;
        }
        let bl = &self.bin_live;
        let lay = &self.layout;

        // 3rd order.
        let (mut b2_max, mut top_bi, mut bi_hits) = (0.0f64, 0usize, 0u32);
        let (mut im_sq_sum, mut im_signed, mut b2_sum) = (0.0, 0.0, 0.0);
        let (mut sig_sin_sum, mut sig_sin_cnt) = (0.0, 0u32);
        for k in 0..nt {
            let (nr, ni) = (self.nbi_re[k], self.nbi_im[k]);
            let den = self.du_bi[k] * self.dv_bi[k];
            let live =
                bl[lay.bi_i1[k] as usize] && bl[lay.bi_i2[k] as usize] && bl[lay.bi_i3[k] as usize];
            let b2 = if den > 1e-30 && live {
                ((nr * nr + ni * ni) / den).min(1.0)
            } else {
                0.0
            };
            let psi = ni.atan2(nr);
            self.b2[k] = b2 as f32;
            self.biphase[k] = psi as f32;
            let s = psi.sin();
            im_sq_sum += b2 * s * s;
            im_signed += b2 * s;
            b2_sum += b2;
            if b2 >= crit_bi {
                bi_hits += 1;
                sig_sin_sum += s.abs();
                sig_sin_cnt += 1;
            }
            if b2 > b2_max {
                b2_max = b2;
                top_bi = k;
            }
        }
        let im_rms_raw = (im_sq_sum / nt as f64).sqrt();
        // Under H0 each b² ≈ Exp-like with mean 1/K and ψ uniform, so E[b² sin²ψ] = 1/(2K).
        let im_bias = if k3 > 1.0 { 0.5 / k3 } else { 0.5 };
        let im_rms = (im_sq_sum / nt as f64 - im_bias).max(0.0).sqrt();
        let im_polarity = if b2_sum > 1e-12 {
            im_signed / b2_sum
        } else {
            0.0
        };
        let sig_im_mean = if sig_sin_cnt > 0 {
            sig_sin_sum / sig_sin_cnt as f64
        } else {
            0.0
        };

        // 4th order + max projection onto (i1, i2).
        self.tri_proj_val.fill(0.0);
        self.tri_proj_phase.fill(0.0);
        self.tri_proj_best_i3.fill(0);
        self.tri_proj_sig.fill(false);
        let (mut t2_max, mut top_tri, mut tri_hits) = (0.0f64, 0usize, 0u32);
        for k in 0..nq {
            let (nr, ni) = (self.ntri_re[k], self.ntri_im[k]);
            let den = self.du_tri[k] * self.dv_tri[k];
            let live = bl[lay.tri_i1[k] as usize]
                && bl[lay.tri_i2[k] as usize]
                && bl[lay.tri_i3[k] as usize]
                && bl[lay.tri_i4[k] as usize];
            let t2 = if den > 1e-30 && live {
                ((nr * nr + ni * ni) / den).min(1.0)
            } else {
                0.0
            };
            let psi = ni.atan2(nr);
            self.t2[k] = t2 as f32;
            self.triphase[k] = psi as f32;
            let sig = t2 >= crit_tri;
            if sig {
                tri_hits += 1;
            }
            if t2 > t2_max {
                t2_max = t2;
                top_tri = k;
            }
            let cell = (lay.tri_i2[k] as usize - 1) * TRI_W + (lay.tri_i1[k] as usize - 1);
            if t2 as f32 >= self.tri_proj_val[cell] {
                self.tri_proj_val[cell] = t2 as f32;
                self.tri_proj_phase[cell] = psi as f32;
                self.tri_proj_best_i3[cell] = lay.tri_i3[k];
            }
            if sig {
                self.tri_proj_sig[cell] = true;
            }
        }

        // Time arrow.
        let phi_rev = if self.m2_dx > 1e-24 {
            self.m3_dx / self.m2_dx.powf(1.5)
        } else {
            0.0
        };
        let p_tot: f64 = self.p_ord.iter().sum();
        let mut d_js = 0.0;
        if p_tot > 1e-30 {
            let inv = 1.0 / p_tot;
            let (mut kl_f, mut kl_b) = (0.0, 0.0);
            for k in 0..64 {
                let pf = self.p_ord[k] * inv;
                let pb = self.p_ord[REV_LUT[k] as usize] * inv;
                let m = 0.5 * (pf + pb);
                if pf > 1e-15 && m > 1e-15 {
                    kl_f += pf * (pf / m).ln();
                }
                if pb > 1e-15 && m > 1e-15 {
                    kl_b += pb * (pb / m).ln();
                }
            }
            d_js = (0.5 * (kl_f + kl_b)).max(0.0);
        }
        let max_peak = self.peak_ring.iter().copied().fold(0.0f32, f32::max) as f64;
        let rms_ema = self.ms_signal.max(1e-30).sqrt();
        let crest = if rms_ema > 1e-9 {
            (max_peak / rms_ema).min(30.0)
        } else {
            0.0
        };

        let l = &self.layout;
        let df = self.df;
        let mut r = Readout {
            k_eff3: k3,
            k_eff4: k4,
            crit_bi,
            crit_tri,
            bi_hits,
            tri_hits,
            b2_max,
            t2_max,
            top_triad: (self.active_hops > 0).then(|| Triad {
                f1: l.bi_i1[top_bi] as f64 * df,
                f2: l.bi_i2[top_bi] as f64 * df,
                f3: l.bi_i3[top_bi] as f64 * df,
                b2: b2_max,
                psi_deg: (self.biphase[top_bi] as f64).to_degrees(),
            }),
            top_quartet: (self.active_hops > 0).then(|| Quartet {
                f1: l.tri_i1[top_tri] as f64 * df,
                f2: l.tri_i2[top_tri] as f64 * df,
                f3: l.tri_i3[top_tri] as f64 * df,
                f4: l.tri_i4[top_tri] as f64 * df,
                t2: t2_max,
                psi_deg: (self.triphase[top_tri] as f64).to_degrees(),
            }),
            phi_rev,
            d_js,
            im_rms_raw,
            im_rms,
            im_polarity,
            sig_im_mean,
            crest,
            h_spec: self.h_spec_ema,
            active_hops: self.active_hops,
            gated: self.is_gated,
            verdict: Verdict::Idle,
            verdict_legacy: Verdict::Idle,
            hop_us: self.last_hop_us,
        };
        r.verdict = classify::classify(&r);
        r.verdict_legacy = classify::classify_legacy(&r);
        self.latest = r;
        r
    }
}
