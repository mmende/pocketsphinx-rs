use std::error::Error;

use crate::{
    fsg::FSG,
    jsgf_rule_iter::{JSGFRule, JSGFRuleIter},
    logmath::LogMath,
};

pub struct JSGF {
    inner: *mut pocketsphinx_sys::jsgf_t,
}

impl JSGF {
    /// Parse a JSGF grammar from a file.
    ///
    /// # Arguments
    /// - `path` - Path to the file to parse.
    /// - `parent` - Optional parent grammar (`None`, usually).
    pub fn from_file(path: &str, parent: Option<&JSGF>) -> Result<Self, Box<dyn Error>> {
        let c_path = std::ffi::CString::new(path)?;
        let parent = parent.map(|p| p.inner).unwrap_or(std::ptr::null_mut());
        let inner = unsafe { pocketsphinx_sys::jsgf_parse_file(c_path.as_ptr(), parent) };
        if inner.is_null() {
            Err("Failed to parse JSGF grammar from file".into())
        } else {
            Ok(Self { inner })
        }
    }

    /// Parse a JSGF grammar from a string.
    ///
    /// # Arguments
    /// - `string` - JSGF string to parse.
    /// - `parent` - Optional parent grammar (`None`, usually).
    pub fn from_string(string: &str, parent: Option<&JSGF>) -> Result<Self, Box<dyn Error>> {
        let c_string = std::ffi::CString::new(string)?;
        let parent = parent.map(|p| p.inner).unwrap_or(std::ptr::null_mut());
        let inner = unsafe { pocketsphinx_sys::jsgf_parse_string(c_string.as_ptr(), parent) };
        if inner.is_null() {
            Err("Failed to parse JSGF grammar from string".into())
        } else {
            Ok(Self { inner })
        }
    }

    /// Get the JSGF grammar name.
    pub fn get_name(&self) -> String {
        let c_str = unsafe { pocketsphinx_sys::jsgf_grammar_name(self.inner) };
        let name = unsafe { std::ffi::CStr::from_ptr(c_str) }
            .to_str()
            .unwrap()
            .to_string();
        name
    }

    /// Get a rule by name from a grammar. Name should not contain brackets.
    pub fn get_rule(&self, name: &str) -> Option<JSGFRule> {
        let c_str = std::ffi::CString::new(name).unwrap();
        let inner = unsafe { pocketsphinx_sys::jsgf_get_rule(self.inner, c_str.as_ptr()) };
        if inner.is_null() {
            None
        } else {
            Some(JSGFRule::from_inner(inner))
        }
    }

    /// Returns the first public rule of the grammar
    pub fn get_public_rule(&self) -> Option<JSGFRule> {
        let inner = unsafe { pocketsphinx_sys::jsgf_get_public_rule(self.inner) };
        if inner.is_null() {
            None
        } else {
            Some(JSGFRule::from_inner(inner))
        }
    }

    /// Get an iterator over all rules in a grammar.
    pub fn get_rule_iter(&self) -> JSGFRuleIter {
        let inner = unsafe { pocketsphinx_sys::jsgf_rule_iter(self.inner) };
        JSGFRuleIter::from_inner(inner)
    }

    /// Build a Sphinx FSG object from a JSGF rule.
    pub fn build_fsg(&self, rule: &JSGFRule, logmath: &LogMath, lw: f32) -> FSG {
        FSG::from_jsgf(self, rule, logmath, lw)
    }

    /// Convert a JSGF rule to Sphinx FSG text form.
    ///
    /// This does a direct conversion without doing transitive closure on null transitions and so forth.
    pub fn write_fsg(&self, rule: &JSGFRule, path: &str) -> Result<(), Box<dyn Error>> {
        let c_path = std::ffi::CString::new(path)?;
        let c_file = unsafe { libc::fopen(c_path.as_ptr(), "rb".as_ptr() as *const i8) };
        if c_file.is_null() {
            return Err("Failed to open fsg output file".into());
        }
        let c_file_ps = c_file as *mut pocketsphinx_sys::FILE;
        let result =
            unsafe { pocketsphinx_sys::jsgf_write_fsg(self.inner, rule.get_inner(), c_file_ps) };
        unsafe { libc::fclose(c_file) };
        // TODO: Check if this is correct (undocumented)
        if result == 0 {
            Ok(())
        } else {
            Err("Failed to write FSG from JSGF grammar".into())
        }
    }

    pub fn get_inner(&self) -> *mut pocketsphinx_sys::jsgf_t {
        self.inner
    }
}

impl Drop for JSGF {
    fn drop(&mut self) {
        unsafe { pocketsphinx_sys::jsgf_grammar_free(self.inner) };
    }
}
