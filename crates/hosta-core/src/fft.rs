//! Exact in-place radix-2 Cooley-Tukey FFT (port of the prototype's `Radix2FFT`).

pub struct Radix2Fft {
    size: usize,
    bit_rev: Vec<u32>,
    cos_table: Vec<f64>,
    sin_table: Vec<f64>,
}

impl Radix2Fft {
    pub fn new(size: usize) -> Self {
        assert!(
            size >= 2 && size.is_power_of_two(),
            "FFT size must be a power of 2 >= 2"
        );
        let bits = size.trailing_zeros();
        let bit_rev = (0..size as u32)
            .map(|i| i.reverse_bits() >> (32 - bits))
            .collect();
        let half = size / 2;
        let (mut cos_table, mut sin_table) = (Vec::with_capacity(half), Vec::with_capacity(half));
        for i in 0..half {
            let ang = -2.0 * std::f64::consts::PI * i as f64 / size as f64;
            cos_table.push(ang.cos());
            sin_table.push(ang.sin());
        }
        Self {
            size,
            bit_rev,
            cos_table,
            sin_table,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// In-place transform. `inverse` conjugates the twiddles and scales by 1/N.
    pub fn transform(&self, re: &mut [f64], im: &mut [f64], inverse: bool) {
        let n = self.size;
        debug_assert!(re.len() >= n && im.len() >= n);
        for i in 0..n {
            let j = self.bit_rev[i] as usize;
            if j > i {
                re.swap(i, j);
                im.swap(i, j);
            }
        }
        let sign = if inverse { -1.0 } else { 1.0 };
        let mut len = 2;
        while len <= n {
            let half_len = len / 2;
            let step = n / len;
            let mut i = 0;
            while i < n {
                let mut tw = 0;
                for j in 0..half_len {
                    let wr = self.cos_table[tw];
                    let wi = sign * self.sin_table[tw];
                    let i1 = i + j;
                    let i2 = i1 + half_len;
                    let (xr, xi) = (re[i2], im[i2]);
                    let tr = xr * wr - xi * wi;
                    let ti = xr * wi + xi * wr;
                    re[i2] = re[i1] - tr;
                    im[i2] = im[i1] - ti;
                    re[i1] += tr;
                    im[i1] += ti;
                    tw += step;
                }
                i += len;
            }
            len <<= 1;
        }
        if inverse {
            let inv = 1.0 / n as f64;
            for v in re[..n].iter_mut().chain(im[..n].iter_mut()) {
                *v *= inv;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_naive_dft_and_roundtrips() {
        let n = 64;
        let fft = Radix2Fft::new(n);
        let x: Vec<f64> = (0..n).map(|i| ((i * 7 % 13) as f64 - 6.0) * 0.1).collect();
        let (mut re, mut im) = (x.clone(), vec![0.0; n]);
        fft.transform(&mut re, &mut im, false);
        for k in 0..n {
            let (mut sr, mut si) = (0.0, 0.0);
            for (t, &v) in x.iter().enumerate() {
                let a = -2.0 * std::f64::consts::PI * (k * t) as f64 / n as f64;
                sr += v * a.cos();
                si += v * a.sin();
            }
            assert!((sr - re[k]).abs() < 1e-9 && (si - im[k]).abs() < 1e-9);
        }
        fft.transform(&mut re, &mut im, true);
        for t in 0..n {
            assert!((re[t] - x[t]).abs() < 1e-12 && im[t].abs() < 1e-12);
        }
    }
}
