use std::ffi::CStr;

pub mod config;
pub mod decoder;
pub mod nbest_iter;
pub mod search_iter;
pub mod seg_iter;

pub fn default_modeldir() -> &'static str {
    unsafe { CStr::from_ptr(pocketsphinx_sys::ps_default_modeldir()) }
        .to_str()
        .unwrap()
}
