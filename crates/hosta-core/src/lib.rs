//! HOSTA core: real-time higher-order spectral and time-arrow analysis.
//!
//! * **3rd order (bicoherence)**: detects quadratic phase coupling `f1 + f2 → f3`,
//!   the signature of asymmetric (even) distortion and sub-to-mid intermodulation.
//! * **4th order (tricoherence)**: detects cubic coupling `f1 + f2 + f3 → f4`,
//!   the signature of symmetric (odd) clipping.
//! * **Time arrow**: lag-1 increment skewness and Jensen-Shannon divergence of
//!   D=3 ordinal-pattern transitions against their time reversal. These separate a
//!   causal hit from a reversed or phase-smeared one, which a magnitude spectrum cannot.
//!
//! Everything runs on a ~8 kHz decimated copy of the input, so the analysis band
//! is 0 – ~3.4 kHz. That covers the sub, low-mid and body region where IMD mud and
//! transient punch live.

pub mod analyzer;
pub mod classify;
pub mod decimator;
pub mod fft;
pub mod spectral;
pub mod synth;

pub use analyzer::{Analyzer, Controls};
pub use classify::Verdict;
pub use spectral::{Readout, SpectralEngine};
