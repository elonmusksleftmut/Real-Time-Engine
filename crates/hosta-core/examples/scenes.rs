//! Print a diagnostic report for every synthetic scene at a given sample rate.
//! `cargo run -p hosta-core --release --example scenes -- 48000`
use hosta_core::synth::{self, Scene};
use hosta_core::{Analyzer, Controls, Verdict};
use std::collections::BTreeMap;

fn run(name: &str, x: &[f32], fs: f64, loops: usize) {
    let mut a = Analyzer::new(fs, 0.6, -55.0);
    let (mut new, mut old) = (BTreeMap::<&str, u32>::new(), BTreeMap::<&str, u32>::new());
    let mut last = None;
    for _ in 0..loops {
        a.process(x, |_, r| {
            if r.active_hops > 30 {
                *new.entry(r.verdict.label()).or_default() += 1;
                *old.entry(r.verdict_legacy.label()).or_default() += 1;
            }
            last = Some(*r);
        });
    }
    let r = last.unwrap();
    let c = Controls::from_readout(&r).as_array();
    println!("{name:<9} NEW {new:?}\n          OLD {old:?}");
    println!(
        "          phi={:+.3} djs={:.4} im={:.3}(raw {:.3}) sigIm={:.2} b2={:.2}/{} t2={:.2}/{} crest={:.2} H={:.3} K3={:.0} crit={:.3}",
        r.phi_rev, r.d_js, r.im_rms, r.im_rms_raw, r.sig_im_mean, r.b2_max, r.bi_hits, r.t2_max, r.tri_hits,
        r.crest, r.h_spec, r.k_eff3, r.crit_bi
    );
    println!("          ctl={:?}", c.map(|v| (v * 100.0).round() / 100.0));
    let _ = Verdict::Clean;
}

fn main() {
    let fs: f64 = std::env::args()
        .nth(1)
        .map(|s| s.parse().unwrap())
        .unwrap_or(48000.0);
    let names = [
        "clean", "oddclip", "tubeIMD", "drykick", "allpass", "reversed", "linphase",
    ];
    for s in Scene::ALL {
        run(names[s as usize], &synth::render_scene(s, fs), fs, 6);
    }
    run("white", &synth::white_noise(fs, 12.0, 0.2, 7), fs, 1);
    run("sine997", &synth::sine(fs, 12.0, 997.0, 0.5), fs, 1);
    run("sine1k", &synth::sine(fs, 12.0, 1000.0, 0.5), fs, 1);
    run("saw110", &synth::saw(fs, 6.0, 110.0, 0.4), fs, 1);
    for f in [62.5, 100.0, 312.0, 440.0] {
        run(&format!("sine{f}"), &synth::sine(fs, 8.0, f, 0.5), fs, 1);
    }
    let mut ns = synth::sine(fs, 8.0, 997.0, 0.5);
    for (a, b) in ns.iter_mut().zip(synth::white_noise(fs, 8.0, 0.0005, 3)) {
        *a += b;
    }
    run("sine+n", &ns, fs, 1);
    let clip: Vec<f32> = synth::sine(fs, 8.0, 110.0, 0.9)
        .iter()
        .map(|v| (3.0 * v).tanh() * 0.5)
        .collect();
    run("sineclip", &clip, fs, 1);
    let asym: Vec<f32> = synth::sine(fs, 8.0, 110.0, 0.5)
        .iter()
        .map(|v| v + 0.3 * v * v)
        .collect();
    run("sinesq", &asym, fs, 1);
}
