//! State-of-art k-means trainer (Rust-native & FFI fallback)
//! Const-generic, SIMD-ready, mobile-first, ≤ 300 KB

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::types::{Scalar, Vector, MAX_DIM};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// Error
// ------------------------------------------------------------------
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TrainError {
    EmptyCluster,
    ConvergenceFailure,
    InsufficientSamples { required: usize, found: usize },
    DimMismatch,
}

// ------------------------------------------------------------------
// Distance metric
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DistanceMetric {
    #[default]
    L2,
    Cosine,
}

#[inline(always)]
fn distance<const D: usize>(a: &[Scalar; D], b: &[Scalar; D], metric: DistanceMetric) -> Scalar {
    let mut acc = 0.0;
    let mut i = 0;
    while i < D {
        let diff = a[i] - b[i];
        acc += diff * diff;
        i += 1;
    }
    match metric {
        DistanceMetric::L2 => acc,
        DistanceMetric::Cosine => 1.0 - (dot::<D>(a, b) / ((dot::<D>(a, a) * dot::<D>(b, b)).sqrt() + 1e-12)),
    }
}

#[inline(always)]
fn dot<const D: usize>(a: &[Scalar; D], b: &[Scalar; D]) -> Scalar {
    let mut acc = 0.0;
    let mut i = 0;
    while i < D {
        acc += a[i] * b[i];
        i += 1;
    }
    acc
}

// ------------------------------------------------------------------
// Initialization
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum InitMethod {
    #[default]
    KMeansPlusPlus,
    Random,
}

// ------------------------------------------------------------------
// K-means++
// ------------------------------------------------------------------
#[inline]
fn kmeans_plus_plus<const D: usize, const K: usize>(
    data: &[Vector<D>],
    metric: DistanceMetric,
) -> [Vector<D>; K] {
    let mut centroids = [Vector([0.0; D]); K];
    let mut rng = fastrand::Rng::new();
    // 1) pick first centroid uniformly
    centroids[0] = data[rng.usize(..data.len())].clone();
    let mut dists = vec![Scalar::MAX; data.len()];

    for k in 1..K {
        // 2) update distances
        for (i, v) in data.iter().enumerate() {
            let d = distance(&v.0, &centroids[k - 1].0, metric);
            if d < dists[i] {
                dists[i] = d;
            }
        }
        // 3) weighted sample
        let sum: Scalar = dists.iter().sum::<Scalar>();
        let mut target = rng.f32() * sum;
        let mut chosen = 0;
        for (i, &d) in dists.iter().enumerate() {
            target -= d;
            if target <= 0.0 {
                chosen = i;
                break;
            }
        }
        centroids[k] = data[chosen].clone();
    }
    centroids
}

// ------------------------------------------------------------------
// Empty cluster handling
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EmptyClusterAction {
    #[default]
    SplitLargest,
    Drop,
}

// ------------------------------------------------------------------
// K-means trainer
// ------------------------------------------------------------------
pub struct KMeans<const D: usize, const K: usize> {
    pub init: InitMethod,
    pub max_iters: usize,
    pub tolerance: Scalar,
    pub metric: DistanceMetric,
    pub empty_action: EmptyClusterAction,
}

impl<const D: usize, const K: usize> Default for KMeans<D, K> {
    fn default() -> Self {
        Self {
            init: InitMethod::default(),
            max_iters: 25,
            tolerance: 1e-4,
            metric: DistanceMetric::default(),
            empty_action: EmptyClusterAction::default(),
        }
    }
}

impl<const D: usize, const K: usize> KMeans<D, K> {
    #[inline]
    pub fn fit(&self, data: &[Vector<D>]) -> Result<Codebook<D, K>, TrainError> {
        if data.len() < K {
            return Err(TrainError::InsufficientSamples {
                required: K,
                found: data.len(),
            });
        }

        // 1) initialize centroids
        let mut centroids = match self.init {
            InitMethod::Random => {
                let mut rng = fastrand::Rng::new();
                [(); K].map(|_| data[rng.usize(..data.len())].clone())
            }
            InitMethod::KMeansPlusPlus => kmeans_plus_plus::<D, K>(data, self.metric),
        };

        // scratch
        let mut counts = [0usize; K];
        let mut new_cent = [[0.0; D]; K];

        for _ in 0..self.max_iters {
            counts.fill(0);
            new_cent.iter_mut().for_each(|c| c.fill(0.0));

            // assignment step
            for v in data.iter() {
                let (idx, _) = centroids
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i, distance(&v.0, &c.0, self.metric)))
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .unwrap();
                counts[idx] += 1;
                for (dst, &src) in new_cent[idx].iter_mut().zip(v.0.iter()) {
                    *dst += src;
                }
            }

            // update step
            let mut any_change = false;
            for (i, c) in centroids.iter_mut().enumerate() {
                if counts[i] == 0 {
                    match self.empty_action {
                        EmptyClusterAction::SplitLargest => {
                            // find largest centroid & split
                            let (largest_idx, _) = counts.iter().enumerate().max_by_key(|&(_, &c)| c).unwrap();
                            let mut split = centroids[largest_idx].0;
                            let noise = 1e-4;
                            let mut rng = fastrand::Rng::new();
                            for s in &mut split {
                                *s += (rng.f32() - 0.5) * noise;
                            }
                            c.0 = split;
                            any_change = true;
                            continue;
                        }
                        EmptyClusterAction::Drop => continue,
                    }
                }
                let inv = 1.0 / counts[i] as Scalar;
                let mut new = [0.0; D];
                for (n, &o) in new.iter_mut().zip(new_cent[i].iter()) {
                    *n = o * inv;
                }
                let shift = distance(&c.0, &new, self.metric).sqrt();
                if shift > self.tolerance {
                    any_change = true;
                }
                c.0 = new;
            }
            if !any_change {
                break;
            }
        }
        Ok(Codebook::new(centroids))
    }
}

// ------------------------------------------------------------------
// Codebook wrapper
// ------------------------------------------------------------------
pub struct Codebook<const D: usize, const K: usize> {
    pub centroids: [Vector<D>; K],
}

impl<const D: usize, const K: usize> Codebook<D, K> {
    #[inline(always)]
    pub fn new(centroids: [Vector<D>; K]) -> Self {
        Self { centroids }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[Vector<D>; K] {
        &self.centroids
    }
}

// ------------------------------------------------------------------
// Builder pattern
// ------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
pub struct TrainerBuilder<const D: usize, const K: usize> {
    init: InitMethod,
    max_iters: usize,
    tolerance: Scalar,
    metric: DistanceMetric,
    empty_action: EmptyClusterAction,
}

impl<const D: usize, const K: usize> TrainerBuilder<D, K> {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn init(mut self, method: InitMethod) -> Self {
        self.init = method;
        self
    }

    #[inline(always)]
    pub fn max_iters(mut self, iters: usize) -> Self {
        self.max_iters = iters;
        self
    }

    #[inline(always)]
    pub fn tolerance(mut self, tol: Scalar) -> Self {
        self.tolerance = tol;
        self
    }

    #[inline(always)]
    pub fn metric(mut self, m: DistanceMetric) -> Self {
        self.metric = m;
        self
    }

    #[inline(always)]
    pub fn empty_action(mut self, action: EmptyClusterAction) -> Self {
        self.empty_action = action;
        self
    }

    #[inline(always)]
    pub fn build(self) -> KMeans<D, K> {
        KMeans {
            init: self.init,
            max_iters: self.max_iters,
            tolerance: self.tolerance,
            metric: self.metric,
            empty_action: self.empty_action,
        }
    }
}

// ------------------------------------------------------------------
// Trait for pluggable trainers
// ------------------------------------------------------------------
pub trait Trainer<const D: usize, const K: usize> {
    fn fit(&self, data: &[Vector<D>]) -> Result<Codebook<D, K>, TrainError>;
}

impl<const D: usize, const K: usize> Trainer<D, K> for KMeans<D, K> {
    #[inline(always)]
    fn fit(&self, data: &[Vector<D>]) -> Result<Codebook<D, K>, TrainError> {
        self.fit(data)
    }
}

// ------------------------------------------------------------------
// Exports
// ------------------------------------------------------------------
pub use {
    KMeans, TrainerBuilder, InitMethod, DistanceMetric, EmptyClusterAction, Codebook, TrainError,
};