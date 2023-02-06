pub struct JSGFRuleIter {
    inner: *mut pocketsphinx_sys::jsgf_rule_iter_t,
    reached_end: bool,
    is_initial: bool,
}

impl JSGFRuleIter {
    pub fn from_inner(inner: *mut pocketsphinx_sys::jsgf_rule_iter_t) -> Self {
        Self {
            inner,
            reached_end: false,
            is_initial: true,
        }
    }

    pub fn rule(&self) -> JSGFRule {
        let rule_inner = unsafe { pocketsphinx_sys::jsgf_rule_iter_rule(self.inner) };
        JSGFRule { inner: rule_inner }
    }
}

impl Iterator for JSGFRuleIter {
    type Item = JSGFRule;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip the first call to jsgf_rule_iter_next in order to get the first rule
        if self.is_initial {
            self.is_initial = false;
        } else {
            self.inner = unsafe { pocketsphinx_sys::jsgf_rule_iter_next(self.inner) };
        }

        if self.reached_end {
            return None;
        }
        if self.inner.is_null() {
            self.reached_end = true;
            return None;
        }

        let rule_inner = unsafe { pocketsphinx_sys::jsgf_rule_iter_rule(self.inner) };
        if rule_inner.is_null() {
            self.reached_end = true;
            return None;
        }
        Some(JSGFRule { inner: rule_inner })
    }
}

impl Drop for JSGFRuleIter {
    fn drop(&mut self) {
        if !self.reached_end {
            unsafe { pocketsphinx_sys::jsgf_rule_iter_free(self.inner) };
        }
    }
}

pub struct JSGFRule {
    inner: *mut pocketsphinx_sys::jsgf_rule_t,
}

/// Rule in a parsed JSGF grammar.
impl JSGFRule {
    pub fn from_inner(inner: *mut pocketsphinx_sys::jsgf_rule_t) -> Self {
        Self { inner }
    }

    /// Get the rule name from a rule.
    pub fn get_name(&self) -> String {
        let c_str = unsafe { pocketsphinx_sys::jsgf_rule_name(self.inner) };
        unsafe { std::ffi::CStr::from_ptr(c_str) }
            .to_str()
            .unwrap()
            .to_string()
    }

    /// Test if a rule is public or not.
    pub fn is_public(&self) -> bool {
        unsafe { pocketsphinx_sys::jsgf_rule_public(self.inner) != 0 }
    }

    pub fn get_inner(&self) -> *mut pocketsphinx_sys::jsgf_rule_t {
        self.inner
    }
}
