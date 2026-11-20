//! Compare the Rust engine (legacy mode) against readouts dumped by `tools/parity.js`.
use hosta_core::spectral::EngineOptions;
use hosta_core::Analyzer;

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("usage: parity <dump_dir> [fs]");
    let fs: f64 = std::env::args()
        .nth(2)
        .map(|s| s.parse().unwrap())
        .unwrap_or(48000.0);
    let names = [
        "b2Max", "t2Max", "biHits", "triHits", "phiRev", "dJs", "imRms", "crest", "hSpec",
        "verdict",
    ];
    let mut all_ok = true;
    for scene in 0..7 {
        let bytes = std::fs::read(format!("{dir}/scene{scene}.f32")).unwrap();
        let x: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        let reference: Vec<Vec<f64>> = std::fs::read_to_string(format!("{dir}/scene{scene}.csv"))
            .unwrap()
            .lines()
            .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
            .collect();
        let mut a = Analyzer::with_options(fs, 0.6, -55.0, EngineOptions::LEGACY);
        let mut rust = vec![];
        a.process(&x, |_, r| {
            rust.push(vec![
                r.b2_max,
                r.t2_max,
                r.bi_hits as f64,
                r.tri_hits as f64,
                r.phi_rev,
                r.d_js,
                r.im_rms_raw,
                r.crest,
                r.h_spec,
                r.verdict_legacy.code() as f64,
            ])
        });
        assert_eq!(rust.len(), reference.len(), "scene {scene}: readout count");
        let mut max_err = [0.0f64; 10];
        let mut mismatched_counts = [0usize; 10];
        // Row 0 is a single frame: every coherence is exactly 1.0 against a
        // threshold of exactly 1.0, so hit counts there are a rounding tie.
        for (row, (a, b)) in rust.iter().zip(&reference).enumerate().skip(1) {
            if (a[2] != b[2] || a[3] != b[3]) && std::env::var("PARITY_DEBUG").is_ok() {
                println!(
                    "  row {row}: rust hits {}/{} vs js {}/{} (b2max {} vs {})",
                    a[2], a[3], b[2], b[3], a[0], b[0]
                );
            }
            for k in 0..10 {
                let e = (a[k] - b[k]).abs() / b[k].abs().max(1e-3);
                max_err[k] = max_err[k].max(e);
                if k == 2 || k == 3 || k == 9 {
                    mismatched_counts[k] += (a[k] != b[k]) as usize;
                }
            }
        }
        let cont_ok = [0, 1, 4, 5, 6, 7, 8].iter().all(|&k| max_err[k] < 1e-6);
        let disc_ok = mismatched_counts.iter().all(|&c| c == 0);
        all_ok &= cont_ok && disc_ok;
        println!(
            "scene {scene}: {} readouts  max rel err [{}]  discrete mismatches: hits3={} hits4={} verdict={}  {}",
            rust.len(),
            [0, 1, 4, 5, 6, 7, 8].iter().map(|&k| format!("{}={:.1e}", names[k], max_err[k])).collect::<Vec<_>>().join(" "),
            mismatched_counts[2], mismatched_counts[3], mismatched_counts[9],
            if cont_ok && disc_ok { "OK" } else { "DIFF" }
        );
    }
    std::process::exit(if all_ok { 0 } else { 1 });
}
