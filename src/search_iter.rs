use crate::decoder::Decoder;

pub struct SearchIter {
    inner: *mut pocketsphinx_sys::ps_search_iter_t,
    reached_end: bool,
    is_initial: bool,
}

impl SearchIter {
    pub fn from_decoder(decoder: &Decoder) -> Self {
        let inner = unsafe { pocketsphinx_sys::ps_search_iter(decoder.get_inner()) };
        Self {
            inner,
            reached_end: false,
            is_initial: true,
        }
    }
}

impl Iterator for SearchIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip the first call to ps_search_iter_next in order to get the first search
        if self.is_initial {
            self.is_initial = false;
        } else {
            self.inner = unsafe { pocketsphinx_sys::ps_search_iter_next(self.inner) };
        }
        if self.reached_end {
            return None;
        }
        if self.inner.is_null() {
            self.reached_end = true;
            return None;
        }

        let c_name = unsafe { pocketsphinx_sys::ps_search_iter_val(self.inner) };
        if c_name.is_null() {
            self.reached_end = true;
            None
        } else {
            let name = unsafe { std::ffi::CStr::from_ptr(c_name) }
                .to_str()
                .unwrap()
                .to_string();
            Some(name)
        }
    }
}

impl Drop for SearchIter {
    fn drop(&mut self) {
        if !self.reached_end {
            unsafe { pocketsphinx_sys::ps_search_iter_free(self.inner) };
        }
    }
}
