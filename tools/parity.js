// Parity harness: runs the browser prototype's RollingSpectralEngine (extracted
// verbatim from time.py) on the 7 synth scenes and dumps the input samples plus
// per-hop readouts, so the Rust port can be checked against it.
//   node tools/parity.js <out_dir> [fs=48000]
// then: cargo run -p hosta-core --release --example parity -- <out_dir>
const fs = require("fs");
const path = require("path");
const src = fs.readFileSync(path.join(__dirname, "..", "time.py"), "utf8");
const grab = (start, end) => {
  const a = src.indexOf(start), b = src.indexOf(end, a);
  if (a < 0 || b < 0) throw new Error("marker not found: " + start);
  return src.slice(a, b);
};
const code =
  grab("class Radix2FFT", "// =====") +
  grab("function designButterworth4LP", "// =====") +
  grab("function revOrd3Code", "// =====") +
  grab("class RollingSpectralEngine", "// =====") +
  grab("function createMulberry32", "// =====");
global.performance = require("perf_hooks").performance;
const M = new Function(code + "; return { RollingSpectralEngine, buildSynthAudioBuffer, designButterworth4LP };")();

const outDir = process.argv[2];
const fsN = +(process.argv[3] || 48000);
fs.mkdirSync(outDir, { recursive: true });
const ctx = { sampleRate: fsN, createBuffer: (c, n) => { const d = new Float32Array(n); return { getChannelData: () => d }; } };
const D = Math.max(1, Math.round(fsN / 8000)), fsA = fsN / D;
const [s0, s1] = M.designButterworth4LP(fsN, 0.42 * fsA);
const VCODE = { "IDLE / STANDBY": 0, "WARMING UP...": 1, "CLEAN / UNCOUPLED": 2, "EVEN / IMD (3rd)": 3, "ODD CLIP (4th)": 4, "CAUSAL PUNCH": 5, "REVERSED ARROW": 6, "PHASE SMEARED": 7 };
for (let scene = 0; scene < 7; scene++) {
  const x = M.buildSynthAudioBuffer(ctx, scene).getChannelData(0);
  const loops = 3;
  const input = new Float32Array(x.length * loops);
  for (let l = 0; l < loops; l++) input.set(x, l * x.length);
  fs.writeFileSync(path.join(outDir, `scene${scene}.f32`), Buffer.from(input.buffer));
  const eng = new M.RollingSpectralEngine(fsA, 0.6, -55);
  let z01 = 0, z02 = 0, z11 = 0, z12 = 0, dc = 0, op = 0;
  const chunk = new Float32Array(64);
  const rows = [];
  for (let i = 0; i < input.length; i++) {
    const xi = input[i];
    const y1 = s0.b0 * xi + z01; z01 = s0.b1 * xi - s0.a1 * y1 + z02; z02 = s0.b2 * xi - s0.a2 * y1;
    const y2 = s1.b0 * y1 + z11; z11 = s1.b1 * y1 - s1.a1 * y2 + z12; z12 = s1.b2 * y1 - s1.a2 * y2;
    if (++dc >= D) {
      dc = 0; chunk[op++] = y2;
      if (op === 64) {
        op = 0; eng.pushDecimated(chunk);
        if (eng.totalSamples >= 256 && eng.totalSamples % 128 === 0) {
          const r = eng.computeReadout();
          rows.push([r.b2Max, r.t2Max, r.biHits, r.triHits, r.phiRev, r.dJs, r.imRms, r.crest, r.hSpec, VCODE[r.verdict]].join(","));
        }
      }
    }
  }
  fs.writeFileSync(path.join(outDir, `scene${scene}.csv`), rows.join("\n") + "\n");
  console.log(`scene ${scene}: ${rows.length} readouts`);
}
