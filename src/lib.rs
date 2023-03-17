use std::ffi::CStr;

pub mod alignment_iter;
pub mod config;
pub mod decoder;
pub mod endpointer;
pub mod fsg;
pub mod jsgf;
pub mod jsgf_rule_iter;
pub mod logmath;
pub mod nbest_iter;
pub mod search_iter;
pub mod seg_iter;
pub mod vad;

pub mod ngram;
pub mod ngram_iter;
pub mod ngram_set_iter;

// Reexport all the modules such that they can be accessed via pocketsphinx::*
pub use alignment_iter::*;
pub use config::*;
pub use decoder::*;
pub use endpointer::*;
pub use fsg::*;
pub use jsgf::*;
pub use jsgf_rule_iter::*;
pub use logmath::*;
pub use nbest_iter::*;
pub use search_iter::*;
pub use seg_iter::*;
pub use vad::*;

pub use ngram::*;
pub use ngram_iter::*;
pub use ngram_set_iter::*;

pub fn default_modeldir() -> &'static str {
    unsafe { CStr::from_ptr(pocketsphinx_sys::ps_default_modeldir()) }
        .to_str()
        .unwrap()
}
