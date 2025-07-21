//! State-of-art preprocessing pipeline for vector quantization
//! Const-generic, SIMD-friendly, mobile-first, ≤ 300 KB

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::types::{Scalar, Vector, QuantizedVector, MAX_DIM};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// Error taxonomy
// ------------------------------------------------------------------
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PreprocessError {
    InsufficientSamples { needed: usize, got: usize },
    SingularCovariance,
    DimMismatch { expected: usize, found: usize },
}

// ------------------------------------------------------------------
// Safe L2 with epsilon
// ------------------------------------------------------------------
#[inline(always)]
fn safe_l2_norm<const D: usize>(v: &[Scalar; D]) -> Scalar {
    let mut sum = 0.0;
    let mut i = 0;
    while i < D {
        sum += v[i] * v[i];
        i += 1;
    }
    (sum + 1e-12).sqrt()
}

/// L2-normalize a vector in-place (returns new instance).
#[derive(Debug, Clone, Copy, Default)]
pub struct L2Norm;

impl L2Norm {
    #[inline(always)]
    pub fn apply<const D: usize>(&self, v: &Vector<D>) -> Vector<D> {
        let norm = safe_l2_norm(&v.0);
        let mut out = [0.0; D];
        let mut i = 0;
        while i < D {
            out[i] = v.0[i] / norm;
            i += 1;
        }
        Vector(out)
    }
}

// ------------------------------------------------------------------
// Mean-centering
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
pub struct MeanCenter {
    pub enabled: bool,
}

impl MeanCenter {
    #[inline(always)]
    pub fn apply<const D: usize>(&self, v: &Vector<D>) -> Vector<D> {
        if !self.enabled {
            return v.clone();
        }
        let mut mean = 0.0;
        let mut i = 0;
        while i < D {
            mean += v.0[i];
            i += 1;
        }
        mean /= D as Scalar;
        let mut out = [0.0; D];
        let mut i = 0;
        while i < D {
            out[i] = v.0[i] - mean;
            i += 1;
        }
        Vector(out)
    }
}

// ------------------------------------------------------------------
// Covariance-SVD PCA
// ------------------------------------------------------------------
pub struct Pca<const IN: usize, const OUT: usize> {
    mean: [Scalar; IN],
    components: Box<[[Scalar; IN]; OUT]>,
}

impl<const IN: usize, const OUT: usize> Pca<IN, OUT> {
    /// Fit PCA via covariance-SVD on a batch of vectors.
    pub fn fit(batch: &[Vector<IN>]) -> Result<Self, PreprocessError> {
        let n = batch.len();
        if n < OUT + 1 {
            return Err(PreprocessError::InsufficientSamples {
                needed: OUT + 1,
                got: n,
            });
        }

        // Mean
        let mut mean = [0.0; IN];
        for v in batch.iter() {
            for (m, &x) in mean.iter_mut().zip(v.0.iter()) {
                *m += x;
            }
        }
        for m in &mut mean {
            *m /= n as Scalar;
        }

        // Centered data
        let mut centered = Vec::with_capacity(n);
        for v in batch.iter() {
            let mut c = [0.0; IN];
            for (ci, (&xi, &mi)) in c.iter_mut().zip(v.0.iter().zip(mean.iter())) {
                *ci = xi - mi;
            }
            centered.push(c);
        }

        // Covariance
        let mut cov = [[0.0; IN]; IN];
        for c in &centered {
            for i in 0..IN {
                for j in 0..IN {
                    cov[i][j] += c[i] * c[j];
                }
            }
        }
        for row in &mut cov {
            for x in row.iter_mut() {
                *x /= n as Scalar;
            }
        }

        // Simple power-iteration top-K (placeholder for SVD)
        let mut components = [[0.0; IN]; OUT];
        for k in 0..OUT {
            let mut v = [0.0; IN];
            // Random init
            for vi in &mut v {
                *vi = fastrand::f32();
            }
            // 10 iterations
            for _ in 0..10 {
                let mut new_v = [0.0; IN];
                for i in 0..IN {
                    for j in 0..IN {
                        new_v[i] += cov[i][j] * v[j];
                    }
                }
                let norm = (new_v.iter().map(|x| x * x).sum::<Scalar>() + 1e-12).sqrt();
                for (vi, ni) in v.iter_mut().zip(new_v.iter()) {
                    *vi = *ni / norm;
                }
            }
            components[k] = v;

            // Deflate
            for c in &centered {
                let proj: Scalar = c.iter().zip(v.iter()).map(|(ci, vi)| ci * vi).sum();
                for (ci, vi) in c.iter().zip(v.iter()) {
                    let idx = c.as_ptr() as usize - centered.as_ptr() as usize;
                    unsafe {
                        let ptr = centered.as_mut_ptr().add(idx / core::mem::size_of::<[Scalar; IN]>()) as *mut Scalar;
                        *ptr = ci - proj * vi;
                    }
                }
            }
        }

        Ok(Self { mean, components: Box::new(components) })
    }

    /// Project down to lower dimension.
    #[inline(always)]
    pub fn project<const OUT: usize>(&self, v: &Vector<IN>) -> Vector<OUT> {
        let mut out = [0.0; OUT];
        let centered = {
            let mut c = [0.0; IN];
            for (ci, (&xi, &mi)) in c.iter_mut().zip(v.0.iter().zip(self.mean.iter())) {
                *ci = xi - mi;
            }
            c
        };
        for (k, comp) in self.components.iter().enumerate() {
            out[k] = centered.iter().zip(comp.iter()).map(|(ci, vi)| ci * vi).sum();
        }
        Vector(out)
    }
}

// ------------------------------------------------------------------
// PCA-Whitening (diagonal)
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
pub struct Whiten {
    pub enabled: bool,
}

impl Whiten {
    #[inline(always)]
    pub fn apply<const D: usize>(&self, v: &Vector<D>) -> Vector<D> {
        if !self.enabled {
            return v.clone();
        }
        let mut out = [0.0; D];
        let mut i = 0;
        while i < D {
            out[i] = v.0[i] / (safe_l2_norm(&v.0) + 1e-12);
            i += 1;
        }
        Vector(out)
    }
}

// ------------------------------------------------------------------
// Fluent builder
// ------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Preprocessor<const D: usize> {
    pub l2: L2Norm,
    pub center: MeanCenter,
    pub whiten: Whiten,
}

impl<const D: usize> Preprocessor<D> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            l2: L2Norm,
            center: MeanCenter { enabled: true },
            whiten: Whiten::default(),
        }
    }

    #[inline(always)]
    pub fn l2_normalize(mut self, enable: bool) -> Self {
        self.l2 = if enable { L2Norm } else { L2Norm };
        self
    }

    #[inline(always)]
    pub fn mean_center(mut self, enable: bool) -> Self {
        self.center.enabled = enable;
        self
    }

    #[inline(always)]
    pub fn whiten(mut self, enable: bool) -> Self {
        self.whiten.enabled = enable;
        self
    }

    #[inline(always)]
    pub fn apply(&self, v: &Vector<D>) -> Vector<D> {
        let v = self.center.apply(v);
        let v = self.l2.apply(&v);
        self.whiten.apply(&v)
    }
}

// ------------------------------------------------------------------
// Functional helpers
// ------------------------------------------------------------------
#[inline(always)]
pub fn l2_normalize<const D: usize>(v: &Vector<D>) -> Vector<D> {
    L2Norm.apply(v)
}

#[inline(always)]
pub fn mean_center<const D: usize>(v: &Vector<D>) -> Vector<D> {
    MeanCenter { enabled: true }.apply(v)
}

#[inline(always)]
pub fn whiten<const D: usize>(v: &Vector<D>) -> Vector<D> {
    Whiten { enabled: true }.apply(v)
}

// ------------------------------------------------------------------
// Trait for pluggable processors
// ------------------------------------------------------------------
pub trait Processor<const D: usize> {
    fn process(&self, v: &Vector<D>) -> Vector<D>;
}

impl<const D: usize> Processor<D> for Preprocessor<D> {
    #[inline(always)]
    fn process(&self, v: &Vector<D>) -> Vector<D> {
        self.apply(v)
    }
}