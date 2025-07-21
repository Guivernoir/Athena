// ffi.rs
// Zero-copy, panic-safe, Send+Sync bindings to faiss_wrapper
// Supports iOS 13+, Android 7.0, macOS, Linux, Windows

#![allow(non_camel_case_types)]
use crate::types::{Scalar, Vector, QuantizedVector, DIM, CODE_BYTES};
use core::ffi::{c_int, c_ulong, c_void};
use core::panic::{catch_unwind, UnwindSafe};
use core::ptr::NonNull;

// ------------------------------------------------------------------
// C opaque handle
// ------------------------------------------------------------------
#[repr(C)]
pub struct faiss_context_t(c_void);

// ------------------------------------------------------------------
// Thin C declarations 
// ------------------------------------------------------------------
extern "C" {
    fn faiss_create(d: c_int) -> *mut faiss_context_t;
    fn faiss_free(ctx: *mut faiss_context_t);

    fn faiss_train(
        ctx: *mut faiss_context_t,
        vectors: *const Scalar,
        n_vectors: c_ulong,
    ) -> c_int;

    fn faiss_encode(
        ctx: *mut faiss_context_t,
        vector: *const Scalar,
        out_codes: *mut u8,
    ) -> c_ulong;

    fn faiss_decode(
        ctx: *mut faiss_context_t,
        codes: *const u8,
        out_vector: *mut Scalar,
    ) -> c_ulong;

    fn faiss_set_omp_num_threads(n: c_int);
    fn faiss_get_omp_max_threads() -> c_int;
}

// ------------------------------------------------------------------
// Error mapping
// ------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TrainError {
    BadDimension = -1,
    Other        = -2,
}

impl From<c_int> for TrainError {
    #[inline(always)]
    fn from(code: c_int) -> Self {
        match code {
            -1 => TrainError::BadDimension,
            _  => TrainError::Other,
        }
    }
}

// ------------------------------------------------------------------
// Safe wrapper (per-instance context)
// ------------------------------------------------------------------
pub struct Quantizer {
    ptr: NonNull<faiss_context_t>,
}

unsafe impl Send for Quantizer {}
unsafe impl Sync for Quantizer {}

impl Quantizer {
    /// Creates a new FAISS context for the **current** `DIM`.
    #[inline]
    pub fn new() -> Result<Self, TrainError> {
        let ptr = unsafe { faiss_create(DIM as c_int) };
        NonNull::new(ptr)
            .map(|ptr| Self { ptr })
            .ok_or(TrainError::BadDimension)
    }

    /// Trains the quantizer on a batch of **contiguous** vectors.
    ///
    /// Accepts `&[Vector<DIM>]` → flattened to a single slice.
    #[inline]
    pub fn train(&mut self, batch: &[Vector<DIM>]) -> Result<(), TrainError> {
        let flat = unsafe {
            // SAFETY: `Vector<DIM>` is `repr(transparent)` over `[Scalar; DIM]`
            slice::from_raw_parts(
                batch.as_ptr() as *const Scalar,
                batch.len() * DIM,
            )
        };
        let n_vectors = batch.len() as c_ulong;
        let ret = unsafe {
            catch_unwind(|| {
                faiss_train(self.ptr.as_ptr(), flat.as_ptr(), n_vectors)
            })
        };
        match ret {
            Ok(0) => Ok(()),
            Ok(c) => Err(TrainError::from(c)),
            Err(_) => Err(TrainError::Other),
        }
    }

    /// Encodes a **single** `Vector<DIM>` → `QuantizedVector`.
    #[inline]
    pub fn encode(&self, v: &Vector<DIM>) -> Result<QuantizedVector, TrainError> {
        let mut out = QuantizedVector::default();
        let written = unsafe {
            catch_unwind(|| {
                faiss_encode(
                    self.ptr.as_ptr(),
                    v.as_slice().as_ptr(),
                    out.as_bytes().as_mut_ptr(),
                )
            })
        };
        match written {
            Ok(CODE_BYTES) => Ok(out),
            _ => Err(TrainError::Other),
        }
    }

    /// Decodes a `QuantizedVector` → `Vector<DIM>`.
    #[inline]
    pub fn decode(&self, q: &QuantizedVector) -> Result<Vector<DIM>, TrainError> {
        let mut out = Vector([0.0; DIM]);
        let written = unsafe {
            catch_unwind(|| {
                faiss_decode(
                    self.ptr.as_ptr(),
                    q.as_bytes().as_ptr(),
                    out.as_mut_slice().as_mut_ptr(),
                )
            })
        };
        match written {
            Ok(DIM) => Ok(out),
            _ => Err(TrainError::Other),
        }
    }

    /// Global thread-pool knob.
    #[inline(always)]
    pub fn set_threads(n: u32) {
        unsafe { faiss_set_omp_num_threads(n as c_int) };
    }

    #[inline(always)]
    pub fn max_threads() -> u32 {
        unsafe { faiss_get_omp_max_threads() as u32 }
    }
}

impl Drop for Quantizer {
    #[inline]
    fn drop(&mut self) {
        unsafe { faiss_free(self.ptr.as_ptr()) };
    }
}

// ------------------------------------------------------------------
// Build-script glue 
// ------------------------------------------------------------------
#[cfg(not(feature = "docs-rs"))]
mod build {
    use std::env;
    use std::path::PathBuf;

    pub fn main() {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let lib_dir = out_dir.join("lib");

        // Tell rustc to link the static libs we built via CMake
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=faiss_wrapper");

        // Static OpenMP (LLVM) already baked into faiss_wrapper.a
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        println!("cargo:rustc-link-lib=static=omp");

        // iOS and Android already embed OpenMP inside faiss_wrapper.a
    }
}

#[cfg(not(feature = "docs-rs"))]
fn main() {
    build::main();
}