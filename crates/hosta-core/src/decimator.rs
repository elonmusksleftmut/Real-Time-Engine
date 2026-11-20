//! Anti-alias 4th-order Butterworth lowpass + integer decimator that feeds the
//! analysis engine with a ~8 kHz stream (port of the prototype's `DecimatorTapProcessor`).

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Biquad {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
}

/// Two cascaded RBJ lowpass sections with exact 4th-order Butterworth pole Qs.
pub fn design_butterworth4_lp(fs_native: f64, fc: f64) -> [Biquad; 2] {
    let fc = fc.max(20.0).min(0.48 * fs_native);
    let w0 = 2.0 * std::f64::consts::PI * fc / fs_native;
    let (sin_w0, cos_w0) = w0.sin_cos();
    // 1/(2 cos(pi/8)), 1/(2 cos(3pi/8))
    let qs = [0.541196100146197, 1.3065629648763766];
    qs.map(|q| {
        let alpha = sin_w0 / (2.0 * q);
        let a0 = 1.0 + alpha;
        let b0 = (1.0 - cos_w0) * 0.5 / a0;
        Biquad {
            b0,
            b1: (1.0 - cos_w0) / a0,
            b2: b0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
        }
    })
}

/// Target analysis rate. All classifier thresholds were tuned on this rate.
pub const TARGET_FS_A: f64 = 8000.0;

pub struct Decimator {
    pub factor: usize,
    pub fs_a: f64,
    s: [Biquad; 2],
    z: [[f64; 2]; 2],
    count: usize,
}

impl Decimator {
    pub fn new(fs_native: f64) -> Self {
        let factor = ((fs_native / TARGET_FS_A).round() as usize).max(1);
        let fs_a = fs_native / factor as f64;
        let s = design_butterworth4_lp(fs_native, 0.42 * fs_a);
        Self {
            factor,
            fs_a,
            s,
            z: [[0.0; 2]; 2],
            count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.z = [[0.0; 2]; 2];
        self.count = 0;
    }

    /// Filter one native-rate sample. Returns `Some(decimated)` every `factor` samples.
    #[inline]
    pub fn push(&mut self, x: f64) -> Option<f64> {
        // Direct Form II Transposed, two sections.
        let [s0, s1] = &self.s;
        let [z0, z1] = &mut self.z;
        let y1 = s0.b0 * x + z0[0];
        z0[0] = s0.b1 * x - s0.a1 * y1 + z0[1];
        z0[1] = s0.b2 * x - s0.a2 * y1;
        let y2 = s1.b0 * y1 + z1[0];
        z1[0] = s1.b1 * y1 - s1.a1 * y2 + z1[1];
        z1[1] = s1.b2 * y1 - s1.a2 * y2;
        self.count += 1;
        if self.count >= self.factor {
            self.count = 0;
            Some(y2)
        } else {
            None
        }
    }
}
