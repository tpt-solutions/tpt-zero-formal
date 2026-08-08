//! A constant-velocity (2-state) Kalman filter implemented entirely on the
//! `no_std` tensor + linalg stack, with no heap allocation.
#![no_std]

use tpt_zero_formal::tensor::Tensor2;

/// State estimate `[position, velocity]` and its 2x2 covariance.
pub struct Kalman2 {
    pub x: Tensor2<f64, 2, 1>,
    pub p: Tensor2<f64, 2, 2>,
}

/// Process model `F` (constant velocity over one time step).
const F: [[f64; 2]; 2] = [[1.0, 1.0], [0.0, 1.0]];

fn identity() -> Tensor2<f64, 2, 2> {
    Tensor2::<f64, 2, 2>::from_fn(|r, c| if r == c { 1.0 } else { 0.0 })
}

impl Kalman2 {
    /// Creates a filter at the origin with the given initial covariance on the
    /// diagonal.
    pub fn new(p0: f64) -> Self {
        Self {
            x: Tensor2::<f64, 2, 1>::from_fn(|_, _| 0.0),
            p: Tensor2::<f64, 2, 2>::from_fn(|r, c| if r == c { p0 } else { 0.0 }),
        }
    }

    /// Time update: `x = F x`, `P = F P F^T + Q`.
    pub fn predict(&mut self, q: f64) {
        let f = Tensor2::<f64, 2, 2>::from_fn(|r, c| F[r][c]);
        self.x = f.mul(&self.x);
        let ft = f.transpose();
        let p = f.mul(&self.p).mul(&ft);
        self.p = p.add(&Tensor2::<f64, 2, 2>::from_fn(|r, c| if r == c { q } else { 0.0 }));
    }

    /// Measurement update with a scalar position observation `z` (variance `r`).
    pub fn update(&mut self, z: f64, r: f64) {
        // Observation model `H` (1x2): we observe position only.
        let h = Tensor2::<f64, 1, 2>::from_fn(|r, c| if r == 0 && c == 0 { 1.0 } else { 0.0 });
        let ht = h.transpose(); // 2x1
        // y = z - H x  (1x1 residual)
        let zv = Tensor2::<f64, 1, 1>::from_fn(|_, _| z);
        let y = zv.sub(&h.mul(&self.x));
        // S = H P H^T + R  (1x1)
        let s = h
            .mul(&self.p)
            .mul(&ht)
            .add(&Tensor2::<f64, 1, 1>::from_fn(|_, _| r));
        // S^-1 (1x1)
        let s_val = *s.get(0, 0).unwrap();
        let inv_s = if s_val.abs() < 1e-12 {
            Tensor2::<f64, 1, 1>::from_fn(|_, _| 0.0)
        } else {
            Tensor2::<f64, 1, 1>::from_fn(|_, _| 1.0 / s_val)
        };
        // K = P H^T S^-1  (2x1)
        let k = self.p.mul(&ht).mul(&inv_s);
        // x = x + K y
        self.x = self.x.add(&k.mul(&y));
        // P = (I - K H) P
        let kh = k.mul(&h); // 2x2
        self.p = identity().sub(&kh).mul(&self.p);
    }

    /// Returns the current position estimate.
    pub fn position(&self) -> f64 {
        *self.x.get(0, 0).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converges_toward_observation() {
        let mut kf = Kalman2::new(1.0);
        for _ in 0..10 {
            kf.predict(0.01);
            kf.update(5.0, 0.1);
        }
        assert!((kf.position() - 5.0).abs() < 0.5);
    }
}
