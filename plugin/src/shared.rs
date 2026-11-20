//! State shared between the audio thread and the editor.
//!
//! The audio thread publishes a [`Snapshot`] after every analysis hop (~62 Hz)
//! with `try_lock`, so it never blocks; if the editor happens to hold the lock,
//! that frame is skipped. All buffers are preallocated, so publishing only copies.

use hosta_core::spectral::{Readout, SpectralEngine, RTA_BINS, SCOPE_LEN, TRI_H, TRI_W};
use hosta_core::Controls;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Number of MIDI CC automation outputs (6 controls + verdict code).
pub const N_OUTPUTS: usize = Controls::COUNT + 1;
pub const OUTPUT_NAMES: [&str; N_OUTPUTS] = [
    "3rd-Order Coupling",
    "4th-Order Coupling",
    "Time Arrow",
    "Irreversibility",
    "Smear",
    "Crest",
    "Verdict Code",
];

#[derive(Clone)]
pub struct Snapshot {
    pub seq: u64,
    pub readout: Readout,
    pub fs_a: f64,
    pub df: f64,
    pub decim: usize,
    pub b2: Vec<f32>,
    pub biphase: Vec<f32>,
    pub tri_val: Vec<f32>,
    pub tri_phase: Vec<f32>,
    pub tri_sig: Vec<bool>,
    pub tri_best_i3: Vec<u16>,
    pub rta_db: Vec<f32>,
    /// Oldest → newest decimated samples.
    pub scope: Vec<f32>,
    /// Smoothed 0..1 values as last sent over MIDI.
    pub outputs: [f32; N_OUTPUTS],
}

impl Snapshot {
    pub fn new(n_triads: usize) -> Self {
        Self {
            seq: 0,
            readout: Readout::default(),
            fs_a: 8000.0,
            df: 8000.0 / 256.0,
            decim: 6,
            b2: vec![0.0; n_triads],
            biphase: vec![0.0; n_triads],
            tri_val: vec![0.0; TRI_W * TRI_H],
            tri_phase: vec![0.0; TRI_W * TRI_H],
            tri_sig: vec![false; TRI_W * TRI_H],
            tri_best_i3: vec![0; TRI_W * TRI_H],
            rta_db: vec![-120.0; RTA_BINS],
            scope: vec![0.0; SCOPE_LEN],
            outputs: [0.0; N_OUTPUTS],
        }
    }

    /// Allocation-free copy of another snapshot (editor side).
    pub fn copy_from(&mut self, o: &Snapshot) {
        self.seq = o.seq;
        self.readout = o.readout;
        self.fs_a = o.fs_a;
        self.df = o.df;
        self.decim = o.decim;
        self.b2.copy_from_slice(&o.b2);
        self.biphase.copy_from_slice(&o.biphase);
        self.tri_val.copy_from_slice(&o.tri_val);
        self.tri_phase.copy_from_slice(&o.tri_phase);
        self.tri_sig.copy_from_slice(&o.tri_sig);
        self.tri_best_i3.copy_from_slice(&o.tri_best_i3);
        self.rta_db.copy_from_slice(&o.rta_db);
        self.scope.copy_from_slice(&o.scope);
        self.outputs = o.outputs;
    }

    /// Copy everything out of the engine. Allocation-free.
    pub fn fill(&mut self, e: &SpectralEngine, decim: usize, outputs: &[f32; N_OUTPUTS]) {
        self.seq += 1;
        self.readout = e.latest;
        self.fs_a = e.fs_a;
        self.df = e.df;
        self.decim = decim;
        self.b2.copy_from_slice(&e.b2);
        self.biphase.copy_from_slice(&e.biphase);
        self.tri_val.copy_from_slice(&e.tri_proj_val);
        self.tri_phase.copy_from_slice(&e.tri_proj_phase);
        self.tri_sig.copy_from_slice(&e.tri_proj_sig);
        self.tri_best_i3.copy_from_slice(&e.tri_proj_best_i3);
        for (d, &p) in self.rta_db.iter_mut().zip(e.rta_power.iter()) {
            *d = (10.0 * p.max(1e-12).log10()) as f32;
        }
        let w = e.scope_write;
        for i in 0..SCOPE_LEN {
            self.scope[i] = e.scope_ring[(w + i) & (SCOPE_LEN - 1)];
        }
        self.outputs = *outputs;
    }
}

pub struct Shared {
    pub snap: Mutex<Snapshot>,
    pub reset_request: AtomicBool,
    /// Host sample rate as f32 bits (for the header telemetry).
    pub sample_rate: AtomicU32,
}

impl Shared {
    pub fn new(n_triads: usize) -> Self {
        Self {
            snap: Mutex::new(Snapshot::new(n_triads)),
            reset_request: AtomicBool::new(false),
            sample_rate: AtomicU32::new(48000f32.to_bits()),
        }
    }

    pub fn sample_rate(&self) -> f32 {
        f32::from_bits(self.sample_rate.load(Ordering::Relaxed))
    }
}
