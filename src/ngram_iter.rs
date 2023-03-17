pub struct NgramIterItem {
    inner: *mut pocketsphinx_sys::ngram_iter_t,
}

/// M-gram (yes, M-gram) iterator object.
///
/// This is an iterator over the N-Gram successors of a given word or N-1-Gram, that is why it is called "M" and not "N".
impl NgramIterItem {
    /// Get information from the current M-gram in an iterator.
    ///
    /// # Returns
    /// (word_ids, score, bowt)
    /// Where word_ids is a vector of word IDs, score is the score for this M-gram (including any word penalty and language weight) and bowt is the backoff weight for this M-gram.
    pub fn get(&self) -> (Vec<i32>, i32, i32) {
        let mut score = 0;
        let mut bowt = 0;
        let word_ids =
            unsafe { pocketsphinx_sys::ngram_iter_get(self.inner, &mut score, &mut bowt) };
        // Convert word_ids to a Vec<i32>
        let mut word_ids_vec = Vec::new();
        let mut i = 0;
        while unsafe { *word_ids.offset(i) } != -1 {
            word_ids_vec.push(unsafe { *word_ids.offset(i) });
            i += 1;
        }
        (word_ids_vec, score, bowt)
    }

    /// Iterate over all M-gram successors of an M-1-gram.
    pub fn successors(&self) -> NgramIter {
        let inner = unsafe { pocketsphinx_sys::ngram_iter_successors(self.inner) };
        NgramIter::from_inner(inner)
    }
}

/// M-gram (yes, M-gram) iterator object.
///
/// This is an iterator over the N-Gram successors of a given word or N-1-Gram, that is why it is called "M" and not "N".
pub struct NgramIter {
    inner: *mut pocketsphinx_sys::ngram_iter_t,
    reached_end: bool,
    is_initial: bool,
}

impl NgramIter {
    pub fn from_inner(inner: *mut pocketsphinx_sys::ngram_iter_t) -> Self {
        Self {
            inner,
            reached_end: false,
            is_initial: true,
        }
    }
}

impl Iterator for NgramIter {
    type Item = NgramIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip the first call to ngram_iter_next in order to get the first ngram
        if self.is_initial {
            self.is_initial = false;
        } else {
            self.inner = unsafe { pocketsphinx_sys::ngram_iter_next(self.inner) };
        }

        if self.reached_end {
            return None;
        }
        if self.inner.is_null() {
            self.reached_end = true;
            return None;
        }

        let ngram = NgramIterItem { inner: self.inner };
        Some(ngram)
    }
}

impl Drop for NgramIter {
    fn drop(&mut self) {
        if !self.reached_end {
            unsafe { pocketsphinx_sys::ngram_iter_free(self.inner) };
        }
    }
}
