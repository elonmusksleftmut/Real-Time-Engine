//! Diagnostic verdict classifier.

use crate::spectral::Readout;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Verdict {
    #[default]
    Idle,
    WarmingUp,
    Clean,
    EvenImd,
    OddClip,
    CausalPunch,
    ReversedArrow,
    PhaseSmeared,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Idle => "IDLE / STANDBY",
            Verdict::WarmingUp => "WARMING UP...",
            Verdict::Clean => "CLEAN / UNCOUPLED",
            Verdict::EvenImd => "EVEN / IMD (3rd)",
            Verdict::OddClip => "ODD CLIP (4th)",
            Verdict::CausalPunch => "CAUSAL PUNCH",
            Verdict::ReversedArrow => "REVERSED ARROW",
            Verdict::PhaseSmeared => "PHASE SMEARED",
        }
    }

    /// Stable numeric code, used for the MIDI "verdict" output.
    pub fn code(self) -> u8 {
        match self {
            Verdict::Idle => 0,
            Verdict::WarmingUp => 1,
            Verdict::Clean => 2,
            Verdict::EvenImd => 3,
            Verdict::OddClip => 4,
            Verdict::CausalPunch => 5,
            Verdict::ReversedArrow => 6,
            Verdict::PhaseSmeared => 7,
        }
    }
}

/// Thresholds of the corrected classifier.
pub mod th {
    /// Increment skewness magnitude that counts as a directional arrow. The
    /// prototype used 0.24; the all-pass-smeared kick sits at |φ| ≈ 0.33 with
    /// D_JS hovering near the 0.0025 line, while dry/reversed hits sit at |φ| ≈ 5.
    pub const PHI_ARROW: f64 = 0.5;
    /// Ordinal irreversibility (nats) needed to *confirm* a direction.
    pub const DJS_ARROW: f64 = 0.0025;
    /// Peak/RMS above which the material is transient-dominated (≈ 14 dB).
    /// Gaussian noise sits near 3.7–4 over the 0.64 s peak window, so this keeps
    /// noise and sustained material out of the transient branch.
    pub const CREST_TRANSIENT: f64 = 5.0;
    /// Crest needed before a skew/irreversibility reading counts as transient *punch*.
    /// A sustained sawtooth is strongly time-asymmetric (φ_rev ≈ +7) but has no hit;
    /// its crest is ≈ 2.1.
    pub const CREST_ARROW: f64 = 3.0;
    pub const COUPLING_HITS: u32 = 3;
    /// Peak coherence needed for a coupling verdict. The clean scene's 99th
    /// percentile is ≈ 0.5 (its tones hold phase for 130 ms, so fewer independent
    /// frames than K_eff assumes); real nonlinear coupling sits at 0.9+.
    pub const COUPLING_MAX: f64 = 0.6;
    pub const WARMUP_HOPS: u64 = 6;
}

/// Corrected classifier.
///
/// The prototype's rules misclassified two of its own seven test scenes:
/// * Clean mix → PHASE SMEARED: `im_rms ≥ 0.06` was below the metric's own noise
///   floor (≈ sqrt(1/2K) ≈ 0.1), so the stationary "smeared" fallback fired on noise.
/// * All-pass smeared kick → REVERSED ARROW: the reverse test accepted
///   `im_rms ≥ 0.05` as an alternative to ordinal irreversibility, and any
///   transient has a large Im[B].
///
/// The fix: direction needs *both* skewness and ordinal irreversibility. Transient
/// material without a confirmed direction is "smeared", and only non-transient
/// material is tested for distortion coupling. On a periodic transient every bin
/// is phase-locked, so bicoherence cannot tell distortion apart there.
pub fn classify(r: &Readout) -> Verdict {
    use th::*;
    if r.active_hops < WARMUP_HOPS {
        return if r.active_hops == 0 {
            Verdict::Idle
        } else {
            Verdict::WarmingUp
        };
    }
    let arrow = r.crest >= CREST_ARROW && r.d_js >= DJS_ARROW && r.phi_rev.abs() >= PHI_ARROW;
    if arrow && r.phi_rev > 0.0 {
        return Verdict::CausalPunch;
    }
    if arrow && r.phi_rev < 0.0 {
        return Verdict::ReversedArrow;
    }
    if r.crest >= CREST_TRANSIENT {
        return Verdict::PhaseSmeared;
    }
    // Quadratic coupling also produces 4th-order *moment* coupling (e.g. (b−a)+a+a → a+b),
    // so the 3rd-order test must win when both fire.
    if r.bi_hits >= COUPLING_HITS && r.b2_max >= COUPLING_MAX {
        return Verdict::EvenImd;
    }
    if r.tri_hits >= COUPLING_HITS && r.t2_max >= COUPLING_MAX {
        return Verdict::OddClip;
    }
    Verdict::Clean
}

/// The prototype's rule set, verbatim, for side-by-side comparison.
pub fn classify_legacy(r: &Readout) -> Verdict {
    if r.active_hops < 6 {
        return if r.active_hops == 0 {
            Verdict::Idle
        } else {
            Verdict::WarmingUp
        };
    }
    let im = r.im_rms_raw;
    if r.phi_rev <= -0.24 && (r.d_js >= 0.0025 || im >= 0.05) {
        Verdict::ReversedArrow
    } else if r.phi_rev >= 0.24 && (r.d_js >= 0.0025 || im >= 0.05) {
        Verdict::CausalPunch
    } else if r.bi_hits >= 3 && r.b2_max >= 0.28 {
        if r.sig_im_mean < 0.34 {
            Verdict::EvenImd
        } else {
            Verdict::PhaseSmeared
        }
    } else if r.tri_hits >= 3 && r.t2_max >= 0.28 {
        Verdict::OddClip
    } else if r.crest >= 2.15 && r.crest <= 3.9 && r.phi_rev.abs() < 0.24 && im >= 0.06 {
        Verdict::PhaseSmeared
    } else {
        Verdict::Clean
    }
}
