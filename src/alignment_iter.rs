use std::ffi::CStr;

use crate::decoder::Decoder;

/// Multi-level alignment (words, phones, states) over an utterance.
pub struct Alignment {
    inner: *mut pocketsphinx_sys::ps_alignment_t,
    retained: bool,
}

impl Alignment {
    /// Create a new alignment from a decoder.
    pub fn from_decoder(decoder: &Decoder) -> Option<Self> {
        let inner = unsafe { pocketsphinx_sys::ps_get_alignment(decoder.get_inner()) };
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                retained: true,
            })
        }
    }

    /// Returns a retained alignment and assures the underlying pointer is not freed before the retained alignment is dropped.
    ///
    /// # Returns
    /// A new alignment with the same underlying pointer.
    pub fn retain(&mut self) -> Self {
        let retained_inner = unsafe { pocketsphinx_sys::ps_alignment_retain(self.inner) };
        self.retained = true;
        Self {
            inner: retained_inner,
            retained: false,
        }
    }

    /// Iterate over the alignment starting at the first word.
    pub fn words(&self) -> AlignmentIter {
        let inner = unsafe { pocketsphinx_sys::ps_alignment_words(self.inner) };
        AlignmentIter::from_inner(inner)
    }

    /// Iterate over the alignment starting at the first phone.
    pub fn phones(&self) -> AlignmentIter {
        let inner = unsafe { pocketsphinx_sys::ps_alignment_phones(self.inner) };
        AlignmentIter::from_inner(inner)
    }

    /// Iterate over the alignment starting at the first state.
    pub fn states(&self) -> AlignmentIter {
        let inner = unsafe { pocketsphinx_sys::ps_alignment_states(self.inner) };
        AlignmentIter::from_inner(inner)
    }

    pub fn get_inner(&self) -> *mut pocketsphinx_sys::ps_alignment_t {
        self.inner
    }
}

impl Drop for Alignment {
    fn drop(&mut self) {
        if !self.retained {
            unsafe { pocketsphinx_sys::ps_alignment_free(self.inner) };
        }
    }
}

pub struct AlignmentIter {
    inner: *mut pocketsphinx_sys::ps_alignment_iter_t,
    reached_end: bool,
    is_initial: bool,
}

impl AlignmentIter {
    pub fn from_inner(inner: *mut pocketsphinx_sys::ps_alignment_iter_t) -> Self {
        Self {
            inner,
            reached_end: false,
            is_initial: true,
        }
    }
}

impl AlignmentIter {
    /// Get the human-readable name of the current segment for an alignment.
    ///
    /// # Returns
    /// Name of this segment as a string (word, phone, or state number).
    pub fn name(&self) -> &str {
        let c_str = unsafe { pocketsphinx_sys::ps_alignment_iter_name(self.inner) };
        let c_str = unsafe { CStr::from_ptr(c_str) };
        c_str.to_str().unwrap()
    }

    /// Get the timing and score information for the current segment of an aligment.
    pub fn seg(&self) -> AlignmentSeg {
        let mut start = 0;
        let mut duration = 0;
        let score = unsafe {
            pocketsphinx_sys::ps_alignment_iter_seg(self.inner, &mut start, &mut duration)
        };
        AlignmentSeg {
            score,
            start,
            duration,
        }
    }

    /// Iterate over the children of the current alignment entry.
    ///
    /// # Returns
    /// An iterator over the children of the current alignment entry or `None` if there are no children.
    pub fn children(&self) -> Option<AlignmentIter> {
        let inner = unsafe { pocketsphinx_sys::ps_alignment_iter_children(self.inner) };
        if inner.is_null() {
            None
        } else {
            Some(AlignmentIter::from_inner(inner))
        }
    }
}

impl Iterator for AlignmentIter {
    type Item = AlignmentIter;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip the first call to ps_alignment_iter_next in order to get the first alignment
        if self.is_initial {
            self.is_initial = false;
        } else {
            self.inner = unsafe { pocketsphinx_sys::ps_alignment_iter_next(self.inner) };
        }

        if self.reached_end {
            return None;
        }
        if self.inner.is_null() {
            self.reached_end = true;
            return None;
        }

        Some(AlignmentIter {
            inner: self.inner,
            reached_end: false,
            is_initial: true,
        })
    }
}

impl Drop for AlignmentIter {
    fn drop(&mut self) {
        if !self.reached_end {
            unsafe { pocketsphinx_sys::ps_alignment_iter_free(self.inner) };
        }
    }
}

pub struct AlignmentSeg {
    /// Acoustic score for this segment
    pub score: i32,
    /// Start frame for this segment
    pub start: i32,
    /// Duration of this segment
    pub duration: i32,
}
