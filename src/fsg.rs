use std::{error::Error, ffi::CString};

use crate::{jsgf::JSGF, jsgf_rule_iter::JSGFRule, logmath::LogMath};

pub struct FSG {
    inner: *mut pocketsphinx_sys::fsg_model_t,
    retained: bool,
}

impl FSG {
    /// Read JSGF from file and return FSG object from it.
    ///
    /// This function looks for a first public rule in jsgf and constructs JSGF from it.
    pub fn from_jsgf_file(path: &str, logmath: &LogMath, lw: f32) -> Self {
        let c_path = CString::new(path).unwrap();
        let inner =
            unsafe { pocketsphinx_sys::jsgf_read_file(c_path.as_ptr(), logmath.get_inner(), lw) };
        Self {
            inner,
            retained: false,
        }
    }

    /// Read JSGF from string and return FSG object from it.
    ///
    /// This function looks for a first public rule in jsgf and constructs JSGF from it.
    pub fn from_jsgf_string(jsgf: &str, logmath: &LogMath, lw: f32) -> Self {
        let c_jsgf = CString::new(jsgf).unwrap();
        let inner =
            unsafe { pocketsphinx_sys::jsgf_read_string(c_jsgf.as_ptr(), logmath.get_inner(), lw) };
        Self {
            inner,
            retained: false,
        }
    }

    /// Build a Sphinx FSG object from a JSGF rule.
    pub fn from_jsgf(jsgf: &JSGF, rule: &JSGFRule, logmath: &LogMath, lw: f32) -> Self {
        let inner = unsafe {
            pocketsphinx_sys::jsgf_build_fsg(
                jsgf.get_inner(),
                rule.get_inner(),
                logmath.get_inner(),
                lw,
            )
        };
        Self {
            inner,
            retained: false,
        }
    }

    /// Returns a ratained FSG and assures the underlying pointer is not freed before the retained FSG is dropped.
    ///
    /// # Returns
    /// A new FSG with the same underlying pointer.
    pub fn retain(&mut self) -> Self {
        let retained_inner = unsafe { pocketsphinx_sys::fsg_model_retain(self.inner) };
        self.retained = true;
        Self {
            inner: retained_inner,
            retained: false,
        }
    }

    /// Read a word FSG from the given file and return a pointer to the structure created. Return NULL if any error occurred.
    ///
    /// File format:
    /// ```
    /// Any number of comment lines; ignored
    /// FSG_BEGIN [<fsgname>]
    /// N <#states>
    /// S <start-state ID>
    /// F <final-state ID>
    /// T <from-state> <to-state> <prob> [<word-string>]
    /// T ...
    /// ... (any number of state transitions)
    /// FSG_END
    /// Any number of comment lines; ignored
    /// ```
    ///
    /// The FSG spec begins with the line containing the keyword FSG_BEGIN. It has an optional fsg name string. If not present, the FSG has the empty string as its name.
    ///
    /// Following the FSG_BEGIN declaration is the number of states, the start state, and the final state, each on a separate line. States are numbered in the range [0 .. <numberofstate>-1].
    ///
    /// These are followed by all the state transitions, each on a separate line, and terminated by the FSG_END line. A state transition has the given probability of being taken, and emits the given word. The word emission is optional; if word-string omitted, it is an epsilon or null transition.
    ///
    /// Comments can also be embedded within the FSG body proper (i.e. between FSG_BEGIN and FSG_END): any line with a # character in col 1 is treated as a comment line.
    ///
    /// # Returns
    /// A new FSG.
    pub fn from_file(path: &str, logmath: &LogMath, lw: f32) -> Result<Self, Box<dyn Error>> {
        let inner = unsafe {
            pocketsphinx_sys::fsg_model_readfile(
                CString::new(path)?.as_ptr(),
                logmath.get_inner(),
                lw,
            )
        };
        if inner.is_null() {
            Err("Failed to read FSG file".into())
        } else {
            Ok(Self {
                inner,
                retained: false,
            })
        }
    }

    /// Check that an FSG accepts a word sequence
    ///
    /// # Arguments
    /// - `words` - Whitespace-separated word sequence
    ///
    /// # Returns
    /// `true` if the FSG accepts the word sequence, `false` otherwise.
    pub fn accept(&self, words: &str) -> bool {
        let c_words = CString::new(words).unwrap();
        let result = unsafe { pocketsphinx_sys::fsg_model_accept(self.inner, c_words.as_ptr()) };
        result == 1
    }

    /// Write FSG to a file.
    ///
    /// # Arguments
    /// - `path` - Path to the file to write to.
    pub fn write_to_file(&self, path: &str) {
        let c_path = CString::new(path).unwrap();
        unsafe { pocketsphinx_sys::fsg_model_writefile(self.inner, c_path.as_ptr()) };
    }

    /// Write FSG to a file in AT&T FSM format.
    ///
    /// # Arguments
    /// - `path` - Path to the file to write to.
    pub fn write_fsm_to_file(&self, path: &str) {
        let c_path = CString::new(path).unwrap();
        unsafe { pocketsphinx_sys::fsg_model_writefile_fsm(self.inner, c_path.as_ptr()) };
    }

    /// Write FSG symbol table to a file (for AT&T FSM)
    ///
    /// # Arguments
    /// - `path` - Path to the file to write to.
    pub fn write_symtab_to_file(&self, path: &str) {
        let c_path = CString::new(path).unwrap();
        unsafe { pocketsphinx_sys::fsg_model_writefile_symtab(self.inner, c_path.as_ptr()) };
    }
}

impl Drop for FSG {
    fn drop(&mut self) {
        if !self.retained {
            unsafe { pocketsphinx_sys::fsg_model_free(self.inner) };
        }
    }
}
