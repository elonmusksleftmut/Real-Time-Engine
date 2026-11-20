//! Native-rate front end: channel fold → anti-alias decimation → fixed-size
//! chunks → [`SpectralEngine`], plus the 0..1 control signals used for automation.

use crate::decimator::Decimator;
use crate::spectral::{EngineOptions, Readout, SpectralEngine, CHUNK};

/// Continuous 0..1 control signals derived from a [`Readout`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Controls {
    /// 3rd-order (even / IMD) coupling above the significance threshold.
    pub coupling3: f64,
    /// 4th-order (odd / symmetric clip) coupling above the significance threshold.
    pub coupling4: f64,
    /// Bipolar time arrow: 0.5 neutral, → 1 forward (causal), → 0 reversed.
    pub arrow: f64,
    /// Ordinal irreversibility strength (D_JS normalised to 0.01 nats).
    pub irreversibility: f64,
    /// Transient-ness × lack of direction: high when a hit has lost its arrow.
    pub smear: f64,
    /// Peak/RMS crest factor, 0 dB → 0, 24 dB → 1.
    pub crest: f64,
}

impl Controls {
    pub const COUNT: usize = 6;
    pub const NAMES: [&'static str; Self::COUNT] = [
        "3rd-Order Coupling",
        "4th-Order Coupling",
        "Time Arrow",
        "Irreversibility",
        "Smear",
        "Crest",
    ];

    pub fn from_readout(r: &Readout) -> Self {
        if r.active_hops < crate::classify::th::WARMUP_HOPS {
            return Controls {
                arrow: 0.5,
                ..Default::default()
            };
        }
        let excess = |v: f64, crit: f64, hits: u32| {
            if hits == 0 || crit >= 1.0 {
                0.0
            } else {
                ((v - crit) / (1.0 - crit)).clamp(0.0, 1.0)
            }
        };
        // Arrow: skewness confirmed by ordinal irreversibility, so a noisy skew
        // with no ordinal asymmetry stays near neutral.
        let confirm = (r.d_js / crate::classify::th::DJS_ARROW).min(1.0);
        let arrow = 0.5 + 0.5 * (r.phi_rev / 1.5).tanh() * confirm;
        let irreversibility = (r.d_js / 0.01).clamp(0.0, 1.0);
        let crest_db = if r.crest > 1e-6 {
            20.0 * r.crest.log10()
        } else {
            0.0
        };
        // Transient-ness ramps over 13..17 dB (crest 4.5 → 7); Gaussian noise sits near 12 dB.
        let transient = ((crest_db - 13.0) / 4.0).clamp(0.0, 1.0);
        let direction = (r.d_js / 0.005).min(1.0) * (r.phi_rev.abs() / 0.5).min(1.0);
        Controls {
            coupling3: excess(r.b2_max, r.crit_bi, r.bi_hits),
            coupling4: excess(r.t2_max, r.crit_tri, r.tri_hits),
            arrow,
            irreversibility,
            smear: transient * (1.0 - direction),
            crest: (crest_db / 24.0).clamp(0.0, 1.0),
        }
    }

    pub fn as_array(&self) -> [f64; Self::COUNT] {
        [
            self.coupling3,
            self.coupling4,
            self.arrow,
            self.irreversibility,
            self.smear,
            self.crest,
        ]
    }
}

/// One-pole DC blocker (~5 Hz) on the decimated stream.
#[derive(Default)]
struct DcBlocker {
    r: f64,
    x1: f64,
    y1: f64,
}

impl DcBlocker {
    #[inline]
    fn push(&mut self, x: f64) -> f64 {
        let y = x - self.x1 + self.r * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
}

pub struct Analyzer {
    pub decimator: Decimator,
    pub engine: SpectralEngine,
    dc: DcBlocker,
    chunk: [f32; CHUNK],
    chunk_pos: usize,
}

impl Analyzer {
    pub fn new(fs_native: f64, tau_sec: f64, gate_db: f64) -> Self {
        let decimator = Decimator::new(fs_native);
        let engine = SpectralEngine::new(decimator.fs_a, tau_sec, gate_db);
        let dc = DcBlocker {
            r: 1.0 - 2.0 * std::f64::consts::PI * 5.0 / decimator.fs_a,
            ..Default::default()
        };
        Self {
            decimator,
            engine,
            dc,
            chunk: [0.0; CHUNK],
            chunk_pos: 0,
        }
    }

    pub fn with_options(fs_native: f64, tau_sec: f64, gate_db: f64, opts: EngineOptions) -> Self {
        let mut a = Self::new(fs_native, tau_sec, gate_db);
        a.engine.set_options(opts);
        a
    }

    pub fn set_options(&mut self, opts: EngineOptions) {
        if opts != self.engine.opts {
            self.engine.set_options(opts);
        }
    }

    pub fn fs_a(&self) -> f64 {
        self.decimator.fs_a
    }

    pub fn set_params(&mut self, tau_sec: f64, gate_db: f64) {
        if tau_sec != self.engine.tau_sec || gate_db != self.engine.gate_db {
            let fs_a = self.decimator.fs_a;
            self.engine.configure(fs_a, tau_sec, gate_db);
        }
    }

    pub fn reset(&mut self) {
        self.decimator.reset();
        self.engine.reset();
        self.dc.x1 = 0.0;
        self.dc.y1 = 0.0;
        self.chunk_pos = 0;
    }

    /// Feed native-rate mono samples. `on_chunk(sample_index, readout)` is called
    /// whenever a 64-sample decimated chunk completed at least one STFT hop;
    /// `sample_index` is the native-rate index within `x` where that happened.
    pub fn process<F: FnMut(usize, &Readout)>(&mut self, x: &[f32], mut on_chunk: F) {
        self.process_engine(x, |i, e| on_chunk(i, &e.latest));
    }

    /// Like [`Analyzer::process`], but hands the callback the whole engine so the
    /// heatmap arrays, RTA and scope can be read right after each readout.
    pub fn process_engine<F: FnMut(usize, &SpectralEngine)>(&mut self, x: &[f32], mut on_chunk: F) {
        for (i, &s) in x.iter().enumerate() {
            if let Some(mut y) = self.decimator.push(s as f64) {
                if !self.engine.opts.legacy {
                    y = self.dc.push(y);
                }
                // f32 storage matches the prototype's Float32Array transport.
                self.chunk[self.chunk_pos] = y as f32;
                self.chunk_pos += 1;
                if self.chunk_pos == CHUNK {
                    self.chunk_pos = 0;
                    let chunk = self.chunk;
                    if self.engine.push_decimated(&chunk) > 0 {
                        self.engine.compute_readout();
                        on_chunk(i, &self.engine);
                    }
                }
            }
        }
    }
}
