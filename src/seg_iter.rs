use crate::{decoder::Decoder, nbest_iter::NBest};

pub struct SegIter {
    inner: *mut pocketsphinx_sys::ps_seg_t,
    reached_end: bool,
    is_initial: bool,
}

impl SegIter {
    pub fn new(decoder: &mut Decoder) -> Option<Self> {
        let inner = unsafe { pocketsphinx_sys::ps_seg_iter(decoder.get_inner()) };
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

    pub fn from_nbest(nbest: &NBest) -> Self {
        let inner = unsafe { pocketsphinx_sys::ps_nbest_seg(nbest.get_inner()) };
        Self {
            inner,
            reached_end: false,
            is_initial: true,
        }
    }
}

impl Iterator for SegIter {
    type Item = Seg;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip the first call to ps_seg_next in order to get the first segment
        if self.is_initial {
            self.is_initial = false;
        } else {
            self.inner = unsafe { pocketsphinx_sys::ps_seg_next(self.inner) };
        }

        if self.reached_end {
            return None;
        }
        if self.inner.is_null() {
            self.reached_end = true;
            return None;
        }

        let seg = Seg { inner: self.inner };
        Some(seg)
    }
}

impl Drop for SegIter {
    fn drop(&mut self) {
        if !self.reached_end {
            unsafe { pocketsphinx_sys::ps_seg_free(self.inner) };
        }
    }
}

pub struct Seg {
    inner: *mut pocketsphinx_sys::ps_seg_t,
}

impl Seg {
    /// Get word string from a segmentation iterator.
    ///
    /// # Returns
    /// Read-only string giving string name of this segment.
    pub fn get_word(&self) -> String {
        let c_word = unsafe { pocketsphinx_sys::ps_seg_word(self.inner) };
        let word = unsafe { std::ffi::CStr::from_ptr(c_word) }
            .to_str()
            .unwrap()
            .to_string();
        word
    }

    /// Get inclusive start and end frames from a segmentation iterator.
    ///
    /// Note: These frame numbers are inclusive, i.e. the end frame refers to the last frame in which the given word or other segment was active.
    /// Therefore, the actual duration is `start` - `end` + 1.
    pub fn get_frames(&self) -> SegFrames {
        let mut start = 0;
        let mut end = 0;
        unsafe { pocketsphinx_sys::ps_seg_frames(self.inner, &mut start, &mut end) };
        SegFrames { start, end }
    }

    /// Get language, acoustic, and posterior probabilities from a segmentation iterator.
    ///
    /// # Returns
    /// Log posterior probability of current segment together with acoustic model score, lm score and lm backoff.
    /// Log is expressed in the log-base used in the decoder.
    /// To convert to linear floating-point, use logmath_exp(`get_logmath()`, pprob).
    pub fn get_prob(&self) -> SegProp {
        let mut am_score = 0;
        let mut lm_score = 0;
        let mut lm_back = 0;
        let prob = unsafe {
            pocketsphinx_sys::ps_seg_prob(self.inner, &mut am_score, &mut lm_score, &mut lm_back)
        };
        SegProp {
            prob,
            am_score,
            lm_score,
            lm_back,
        }
    }
}

pub struct SegFrames {
    /// First frame index in segment.
    pub start: i32,
    /// Last frame index in segment.
    pub end: i32,
}

pub struct SegProp {
    /// Unless the -bestpath option is enabled, this will always be zero (corresponding to a posterior probability of `1.0`).
    /// Even if -bestpath is enabled, it will also return zero when called on a partial result.
    /// Ongoing research into effective confidence annotation for partial hypotheses may result in these restrictions being lifted in future versions.
    pub prob: i32,
    /// Acoustic model score for this segment.
    pub am_score: i32,
    /// Language model score for this segment.
    pub lm_score: i32,
    /// Language model backoff mode for this segment (i.e. the number of words used in calculating lscr).
    /// This field is, of course, only meaningful for N-Gram models.
    pub lm_back: i32,
}
