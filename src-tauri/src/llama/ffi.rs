use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_void};

/**
 * Raw FFI Bindings for LLM Engine
 * 
 * This is where we venture into the unsafe wilderness of C interop.
 * Handle with appropriate tactical caution - one wrong pointer dereference
 * and your entire operation goes sideways faster than a chess match with Kasparov.
 */
#[link(name = "llama")]
extern "C" {
    fn qwen_engine_create(model_path: *const c_char) -> *mut c_void;
    fn qwen_engine_destroy(engine: *mut c_void);
    fn qwen_engine_generate(
        engine: *mut c_void,
        prompt: *const c_char,
        max_tokens: c_int,
        temperature: c_float,
    ) -> *mut c_char;
    fn qwen_engine_chat(
        engine: *mut c_void,
        system_prompt: *const c_char,
        user_message: *const c_char,
        max_tokens: c_int,
    ) -> *mut c_char;
    fn qwen_free_string(str: *mut c_char);
    fn qwen_engine_is_loaded(engine: *mut c_void) -> c_int;
    fn qwen_engine_get_model_info(engine: *mut c_void) -> *const c_char;
}

pub struct RawEngine {
    pub(crate) ptr: *mut c_void,
}

impl RawEngine {
    pub unsafe fn new(model_path: &str) -> Option<Self> {
        let c_path = CString::new(model_path).ok()?;
        let ptr = qwen_engine_create(c_path.as_ptr());
        if ptr.is_null() {
            None
        } else {
            Some(RawEngine { ptr })
        }
    }
    pub unsafe fn generate(
        &self,
        prompt: &str,
        max_tokens: i32,
        temperature: f32,
    ) -> Option<String> {
        let c_prompt = CString::new(prompt).ok()?;
        let result_ptr = qwen_engine_generate(
            self.ptr,
            c_prompt.as_ptr(),
            max_tokens as c_int,
            temperature as c_float,
        );
        if result_ptr.is_null() {
            return None;
        }
        let c_str = CStr::from_ptr(result_ptr);
        let rust_string = c_str.to_string_lossy().into_owned();
        qwen_free_string(result_ptr);
        Some(rust_string)
    }
    pub unsafe fn chat(
        &self,
        system_prompt: Option<&str>,
        user_message: &str,
        max_tokens: i32,
    ) -> Option<String> {
        let c_user = CString::new(user_message).ok()?;
        let c_system = match system_prompt {
            Some(s) => Some(CString::new(s).ok()?),
            None => None,
        };
        let system_ptr = match &c_system {
            Some(s) => s.as_ptr(),
            None => std::ptr::null(),
        };
        let result_ptr = qwen_engine_chat(
            self.ptr,
            system_ptr,
            c_user.as_ptr(),
            max_tokens as c_int,
        );
        if result_ptr.is_null() {
            return None;
        }
        let c_str = CStr::from_ptr(result_ptr);
        let rust_string = c_str.to_string_lossy().into_owned();
        qwen_free_string(result_ptr);
        Some(rust_string)
    }
    pub unsafe fn is_loaded(&self) -> bool {
        qwen_engine_is_loaded(self.ptr) != 0
    }
    pub unsafe fn get_model_info(&self) -> String {
        let info_ptr = qwen_engine_get_model_info(self.ptr);
        if info_ptr.is_null() {
            return "Unknown model status".to_string();
        }
        let c_str = CStr::from_ptr(info_ptr);
        c_str.to_string_lossy().into_owned()
    }
}

impl Drop for RawEngine {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                qwen_engine_destroy(self.ptr);
                self.ptr = std::ptr::null_mut();
            }
        }
    }
}

unsafe impl Send for RawEngine {}

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