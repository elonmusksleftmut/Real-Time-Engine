//! Regression tests: every synthetic scene must land on its intended verdict at
//! common host sample rates, and the audio path must never allocate.

use hosta_core::synth::{self, Scene};
use hosta_core::{Analyzer, Controls, Readout, Verdict};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

const RATES: [f64; 3] = [44100.0, 48000.0, 96000.0];

/// Run `x` `loops` times; return the verdict histogram after warm-up and the last readout.
fn run(x: &[f32], fs: f64, loops: usize) -> (HashMap<Verdict, usize>, usize, Readout) {
    let mut a = Analyzer::new(fs, 0.6, -55.0);
    let mut hist = HashMap::new();
    let mut total = 0;
    let mut last = Readout::default();
    for _ in 0..loops {
        a.process(x, |_, r| {
            // Skip ~1 s of EMA warm-up.
            if r.active_hops > 60 {
                *hist.entry(r.verdict).or_insert(0) += 1;
                total += 1;
            }
            last = *r;
        });
    }
    (hist, total, last)
}

fn share(hist: &HashMap<Verdict, usize>, total: usize, v: Verdict) -> f64 {
    *hist.get(&v).unwrap_or(&0) as f64 / total.max(1) as f64
}

fn expect(name: &str, x: &[f32], fs: f64, loops: usize, want: Verdict, min_share: f64) {
    let (hist, total, last) = run(x, fs, loops);
    let s = share(&hist, total, want);
    assert!(
        total > 50 && s >= min_share,
        "{name} @ {fs} Hz: wanted {want:?} ≥ {min_share}, got {s:.3}. histogram {hist:?}; last {last:#?}"
    );
}

#[test]
fn prototype_scenes_classify_correctly() {
    let want = [
        (Scene::CleanMix, Verdict::Clean, 0.95),
        (Scene::OddClipper, Verdict::OddClip, 0.98),
        (Scene::TubeSubImd, Verdict::EvenImd, 0.98),
        (Scene::DryKick, Verdict::CausalPunch, 0.98),
        (Scene::AllPassKick, Verdict::PhaseSmeared, 0.95),
        (Scene::ReversedKick, Verdict::ReversedArrow, 0.98),
        (Scene::LinearPhaseKick, Verdict::PhaseSmeared, 0.98),
    ];
    for fs in RATES {
        for (scene, v, min) in want {
            let loops = if (scene as u32) <= 2 { 3 } else { 6 };
            expect(
                &format!("{scene:?}"),
                &synth::render_scene(scene, fs),
                fs,
                loops,
                v,
                min,
            );
        }
    }
}

#[test]
fn legacy_rules_reproduce_the_prototype_bugs() {
    // Documents *why* the classifier changed: the prototype's rules call the
    // clean mix "smeared" and the all-pass kick "reversed".
    let fs = 48000.0;
    let legacy = |scene| {
        let mut a = Analyzer::new(fs, 0.6, -55.0);
        let mut h: HashMap<Verdict, usize> = HashMap::new();
        let x = synth::render_scene(scene, fs);
        for _ in 0..3 {
            a.process(&x, |_, r| {
                if r.active_hops > 60 {
                    *h.entry(r.verdict_legacy).or_default() += 1;
                }
            });
        }
        h.into_iter().max_by_key(|(_, n)| *n).unwrap().0
    };
    assert_eq!(legacy(Scene::CleanMix), Verdict::PhaseSmeared);
    assert_eq!(legacy(Scene::AllPassKick), Verdict::ReversedArrow);
}

#[test]
fn test_tones_and_noise_read_clean() {
    for fs in RATES {
        for f in [55.0, 100.0, 312.0, 440.0, 997.0, 1000.0, 2500.0] {
            expect(
                &format!("sine {f}"),
                &synth::sine(fs, 6.0, f, 0.5),
                fs,
                1,
                Verdict::Clean,
                0.99,
            );
        }
        let mut noisy = synth::sine(fs, 6.0, 997.0, 0.5);
        for (a, b) in noisy.iter_mut().zip(synth::white_noise(fs, 6.0, 5e-4, 3)) {
            *a += b;
        }
        expect("sine + -60 dB noise", &noisy, fs, 1, Verdict::Clean, 0.99);
        expect(
            "white noise",
            &synth::white_noise(fs, 8.0, 0.2, 11),
            fs,
            1,
            Verdict::Clean,
            0.97,
        );
    }
}

#[test]
fn single_tone_nonlinearities_are_detected() {
    for fs in RATES {
        let s = synth::sine(fs, 6.0, 110.0, 0.9);
        let clip: Vec<f32> = s.iter().map(|v| (3.0 * v).tanh() * 0.5).collect();
        expect("tanh clip", &clip, fs, 1, Verdict::OddClip, 0.99);
        let asym: Vec<f32> = s.iter().map(|v| 0.6 * v + 0.3 * v * v).collect();
        expect("x + x²", &asym, fs, 1, Verdict::EvenImd, 0.99);
    }
}

#[test]
fn silence_gates_and_freezes() {
    let fs = 48000.0;
    let mut a = Analyzer::new(fs, 0.6, -55.0);
    let mut n = 0;
    a.process(&vec![0.0; 48000], |_, r| {
        assert!(r.gated);
        assert_eq!(r.verdict, Verdict::Idle);
        n += 1;
    });
    assert!(n > 0);
}

#[test]
fn control_outputs_track_the_scenes() {
    let fs = 48000.0;
    let ctl = |scene| {
        let (_, _, r) = run(&synth::render_scene(scene, fs), fs, 4);
        Controls::from_readout(&r)
    };
    let dry = ctl(Scene::DryKick);
    let rev = ctl(Scene::ReversedKick);
    let lin = ctl(Scene::LinearPhaseKick);
    let odd = ctl(Scene::OddClipper);
    let tube = ctl(Scene::TubeSubImd);
    let clean = ctl(Scene::CleanMix);
    assert!(dry.arrow > 0.9 && rev.arrow < 0.1, "arrow {dry:?} {rev:?}");
    assert!(
        (lin.arrow - 0.5).abs() < 0.05,
        "linear-phase arrow should be neutral: {lin:?}"
    );
    assert!(
        lin.smear > 0.8 && dry.smear < 0.1 && rev.smear < 0.1,
        "smear {lin:?} {dry:?}"
    );
    assert!(odd.coupling4 > 0.8 && odd.coupling3 < 0.2, "odd {odd:?}");
    assert!(tube.coupling3 > 0.8, "tube {tube:?}");
    assert!(
        clean.coupling3 < 0.3 && clean.coupling4 < 0.3,
        "clean {clean:?}"
    );
    for c in [dry, rev, lin, odd, tube, clean] {
        assert!(c.as_array().iter().all(|v| (0.0..=1.0).contains(v)));
    }
}

// ---- real-time safety ---------------------------------------------------------

struct CountingAlloc;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
thread_local! {
    // Only count the measuring thread; other tests allocate in parallel.
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if COUNTING.with(|c| c.get()) {
            ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

#[test]
fn process_never_allocates() {
    let fs = 48000.0;
    let x = synth::render_scene(Scene::TubeSubImd, fs);
    let mut a = Analyzer::new(fs, 0.6, -55.0);
    a.process(&x[..4800], |_, _| {}); // warm up outside the measured region
    let mut sink = 0.0;
    COUNTING.with(|c| c.set(true));
    for block in x.chunks(512) {
        a.process(block, |_, r| sink += Controls::from_readout(r).coupling3);
    }
    COUNTING.with(|c| c.set(false));
    assert_eq!(ALLOCS.load(Ordering::SeqCst), 0, "audio path allocated");
    assert!(sink.is_finite());
}
