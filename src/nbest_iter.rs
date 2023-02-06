use crate::{decoder::Decoder, seg_iter::SegIter};

pub struct NBestIter {
    inner: *mut pocketsphinx_sys::ps_nbest_t,
    reached_end: bool,
    is_initial: bool,
}

impl NBestIter {
    pub fn from_decoder(decoder: &Decoder) -> Option<Self> {
        let inner = unsafe { pocketsphinx_sys::ps_nbest(decoder.get_inner()) };
        if inner.is_null() {
            return None;
        } else {
            Some(Self {
                inner,
                reached_end: false,
                is_initial: true,
            })
        }
    }
}

impl Iterator for NBestIter {
    type Item = NBest;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip the first call to ps_nbest_next in order to get the first segment
        if self.is_initial {
            self.is_initial = false;
        } else {
            self.inner = unsafe { pocketsphinx_sys::ps_nbest_next(self.inner) };
        }

        if self.reached_end {
            return None;
        }
        if self.inner.is_null() {
            self.reached_end = true;
            return None;
        }

        let nbest = NBest { inner: self.inner };
        Some(nbest)
    }
}

impl Drop for NBestIter {
    fn drop(&mut self) {
        if !self.reached_end {
            unsafe { pocketsphinx_sys::ps_nbest_free(self.inner) };
        }
    }
}

pub struct NBest {
    inner: *mut pocketsphinx_sys::ps_nbest_t,
}

impl NBest {
    /// Get the hypothesis from an N-best list iterator.
    pub fn get_hyp(&self) -> NBestHypothesis {
        let mut score = 0;
        let c_hyp = unsafe { pocketsphinx_sys::ps_nbest_hyp(self.inner, &mut score) };
        let hypothesis;
        if c_hyp.is_null() {
            hypothesis = "".to_string();
        } else {
            hypothesis = unsafe { std::ffi::CStr::from_ptr(c_hyp) }
                .to_str()
                .unwrap()
                .to_string();
        }
        NBestHypothesis { hypothesis, score }
    }

    /// Get the word segmentation from the N-best.
    pub fn get_seg(&self) -> SegIter {
        SegIter::from_nbest(self)
    }

    pub fn get_inner(&self) -> *mut pocketsphinx_sys::ps_nbest_t {
        self.inner
    }
}

pub struct NBestHypothesis {
    /// Hypothesis string from N-best list iterator.
    pub hypothesis: String,
    /// Path score for this hypothesis.
    pub score: i32,
}
