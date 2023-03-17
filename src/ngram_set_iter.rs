use std::ffi::{c_char, CStr};

use crate::Ngram;

pub struct NgramSetIterItem {
    inner: *mut pocketsphinx_sys::ngram_model_set_iter_t,
}

impl NgramSetIterItem {
    /// Get language model and associated name from an iterator.
    ///
    /// # Returns
    /// (Ngram, lmname)
    /// Where NgramModel is the language model and lmname is the name of the language model.
    pub fn model(&self) -> (Ngram, Option<String>) {
        let mut lmname: *const c_char = std::ptr::null();
        let inner =
            unsafe { pocketsphinx_sys::ngram_model_set_iter_model(self.inner, &mut lmname) };
        let lmname = match unsafe { CStr::from_ptr(lmname).to_str() } {
            Ok(s) => Some(s.to_string()),
            Err(_) => None,
        };
        (Ngram::from_inner(inner), lmname)
    }
}

pub struct NgramSetIter {
    inner: *mut pocketsphinx_sys::ngram_model_set_iter_t,
    reached_end: bool,
    is_initial: bool,
}

impl NgramSetIter {
    pub fn from_inner(inner: *mut pocketsphinx_sys::ngram_model_set_iter_t) -> Self {
        Self {
            inner,
            reached_end: false,
            is_initial: true,
        }
    }
}

impl Iterator for NgramSetIter {
    type Item = NgramSetIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip the first call to ngram_set_iter_next in order to get the first ngram
        if self.is_initial {
            self.is_initial = false;
        } else {
            self.inner = unsafe { pocketsphinx_sys::ngram_model_set_iter_next(self.inner) };
        }

        if self.reached_end {
            return None;
        }
        if self.inner.is_null() {
            self.reached_end = true;
            return None;
        }

        let ngram = NgramSetIterItem { inner: self.inner };
        Some(ngram)
    }
}

impl Drop for NgramSetIter {
    fn drop(&mut self) {
        if !self.reached_end {
            unsafe { pocketsphinx_sys::ngram_model_set_iter_free(self.inner) };
        }
    }
}
