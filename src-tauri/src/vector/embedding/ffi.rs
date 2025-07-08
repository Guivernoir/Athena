use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_void};

/**
 * Raw FFI Bindings for BGE Embedding Engine
 * 
 * Strategic pivot to vectorial operations. We're now in the business
 * of semantic understanding rather than creative expression.
 * Handle with tactical precision - floating point arrays are delicate assets.
 */

#[link(name = "llama")]
extern "C" {
    fn bge_engine_create(model_path: *const c_char) -> *mut c_void;
    fn bge_engine_destroy(engine: *mut c_void);
    fn bge_engine_embed(
        engine: *mut c_void,
        text: *const c_char,
        embedding_size: *mut c_int,
    ) -> *mut c_float;
    fn bge_free_embedding(embedding: *mut c_float);
    fn bge_engine_is_loaded(engine: *mut c_void) -> c_int;
    fn bge_engine_get_embedding_dim(engine: *mut c_void) -> c_int;
    fn bge_engine_get_model_info(engine: *mut c_void) -> *const c_char;
}

pub struct RawEmbeddingEngine {
    pub(crate) ptr: *mut c_void,
}

impl RawEmbeddingEngine {
    pub unsafe fn new(model_path: &str) -> Option<Self> {
        let c_path = CString::new(model_path).ok()?;
        let ptr = bge_engine_create(c_path.as_ptr());
        if ptr.is_null() {
            None
        } else {
            Some(RawEmbeddingEngine { ptr })
        }
    }

    pub unsafe fn embed(&self, text: &str) -> Option<Vec<f32>> {
        let c_text = CString::new(text).ok()?;
        let mut embedding_size: c_int = 0;
        
        let result_ptr = bge_engine_embed(
            self.ptr,
            c_text.as_ptr(),
            &mut embedding_size as *mut c_int,
        );
        
        if result_ptr.is_null() || embedding_size <= 0 {
            return None;
        }
        
        let slice = std::slice::from_raw_parts(result_ptr, embedding_size as usize);
        let embedding: Vec<f32> = slice.to_vec();
        
        bge_free_embedding(result_ptr);
        Some(embedding)
    }

    pub unsafe fn is_loaded(&self) -> bool {
        bge_engine_is_loaded(self.ptr) != 0
    }

    pub unsafe fn get_embedding_dim(&self) -> usize {
        bge_engine_get_embedding_dim(self.ptr) as usize
    }

    pub unsafe fn get_model_info(&self) -> String {
        let info_ptr = bge_engine_get_model_info(self.ptr);
        if info_ptr.is_null() {
            return "Unknown embedding model status".to_string();
        }
        let c_str = CStr::from_ptr(info_ptr);
        c_str.to_string_lossy().into_owned()
    }
}

impl Drop for RawEmbeddingEngine {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                bge_engine_destroy(self.ptr);
                self.ptr = std::ptr::null_mut();
            }
        }
    }
}

unsafe impl Send for RawEmbeddingEngine {}

pub(crate) mod utils {
    use super::*;
    
    pub fn to_c_string(s: &str) -> Result<CString, std::ffi::NulError> {
        CString::new(s)
    }
    
    pub unsafe fn from_c_string(ptr: *const c_char) -> Option<String> {
        if ptr.is_null() {
            return None;
        }
        let c_str = CStr::from_ptr(ptr);
        Some(c_str.to_string_lossy().into_owned())
    }
}