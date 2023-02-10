use std::{error::Error, ffi::CString};

use crate::decoder::Decoder;

pub struct LogMath {
    inner: *mut pocketsphinx_sys::logmath_t,
    retained: bool,
}

impl LogMath {
    /// Initialize a log math computation table.
    ///
    /// # Arguments
    /// - `base` - The base B in which computation is to be done.
    /// - `shift` - Log values are shifted right by this many bits.
    /// - `use_table` - Whether to use an add table or not
    pub fn new(base: f64, shift: i32, use_table: bool) -> Self {
        let use_table = if use_table { 1 } else { 0 };
        let inner = unsafe { pocketsphinx_sys::logmath_init(base, shift, use_table) };
        Self {
            inner,
            retained: false,
        }
    }

    /// Get logmath from decoder.
    pub fn from_decoder(decoder: &Decoder) -> Self {
        let inner = unsafe { pocketsphinx_sys::ps_get_logmath(decoder.get_inner()) };
        Self {
            inner,
            retained: true,
        }
    }

    /// Memory-map (or read) a log table from a file.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#ad5f25906919e112859a51dec5aa96752
    ///
    /// # Arguments
    /// - `path` - Path to the log table file.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let path = CString::new(path)?;
        let inner = unsafe { pocketsphinx_sys::logmath_read(path.as_ptr()) };
        if inner.is_null() {
            Err("Failed to read logmath from file".into())
        } else {
            Ok(Self {
                inner,
                retained: false,
            })
        }
    }

    /// Write a log table to a file.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#a02bcbb922fcbb4983cc003da84f5da6d
    ///
    /// # Arguments
    /// - `path` - Path to the log table file to write to.
    pub fn write_to_file(&self, path: &str) -> Result<i32, Box<dyn Error>> {
        let path = CString::new(path)?;
        Ok(unsafe { pocketsphinx_sys::logmath_write(self.inner, path.as_ptr()) })
    }

    /// Get the log table size and dimensions.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#ae921cd21e0c7c28a5801f5063ddb8228
    pub fn get_table_shape(&self) -> LogMathTableShape {
        let mut size = 0;
        let mut width = 0;
        let mut shift = 0;
        let shape = unsafe {
            pocketsphinx_sys::logmath_get_table_shape(self.inner, &mut size, &mut width, &mut shift)
        };
        LogMathTableShape {
            shape,
            size,
            width,
            shift,
        }
    }

    /// Get the log base.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#a6e619faaacbd3eea427e22b223353754
    pub fn get_base(&self) -> f64 {
        unsafe { pocketsphinx_sys::logmath_get_base(self.inner) }
    }

    /// Get the smallest possible value represented in this base.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#aa7188fd7b15e28688d86a368e31df2e0
    pub fn get_zero(&self) -> i32 {
        unsafe { pocketsphinx_sys::logmath_get_zero(self.inner) }
    }

    /// Get the width of the values in a log table.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#ae484ab0132dc7324ee9927b660ac77d8
    pub fn get_width(&self) -> i32 {
        unsafe { pocketsphinx_sys::logmath_get_width(self.inner) }
    }

    /// Get the shift of the values in a log table.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#af12b536b7cd046d8c02f98b647e290cd
    pub fn get_shift(&self) -> i32 {
        unsafe { pocketsphinx_sys::logmath_get_shift(self.inner) }
    }

    /// Returns a retained log table and assures the underlying log table is only be dropped after the retained log table has been drops.
    ///
    /// # Returns
    /// A new Logmath instance with the same inner pointer.
    pub fn retain(&mut self) -> Self {
        let retained_inner = unsafe { pocketsphinx_sys::logmath_retain(self.inner) };
        self.retained = true;

        Self {
            inner: retained_inner,
            retained: false,
        }
    }

    /// Add two values in log space exactly and slowly (without using add table).
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#ab706b56ac49ab2dfa945d2d32758ab55
    pub fn add_exact(&self, logb_p: i32, logb_q: i32) -> i32 {
        unsafe { pocketsphinx_sys::logmath_add_exact(self.inner, logb_p, logb_q) }
    }

    /// Add two values in log space (i.e. return log(exp(p)+exp(q)))
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#a49c7b087532d7cf136167ad6cf61e4b3
    pub fn add(&self, logb_p: i32, logb_q: i32) -> i32 {
        unsafe { pocketsphinx_sys::logmath_add(self.inner, logb_p, logb_q) }
    }

    /// Convert linear floating point number to integer log in base B.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#a9e3c7cbe0d9b3ba74b7e6965cacd6d5a
    pub fn log(&self, p: f64) -> i32 {
        unsafe { pocketsphinx_sys::logmath_log(self.inner, p) }
    }

    /// Convert integer log in base B to linear floating point.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#a439a78b7c6f5821a750574ececc20fae
    pub fn exp(&self, logb_p: i32) -> f64 {
        unsafe { pocketsphinx_sys::logmath_exp(self.inner, logb_p) }
    }

    /// Convert natural log (in floating point) to integer log in base B.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#a66303cb0b91f43452ea500f53ef7eaa5
    pub fn ln_to_log(&self, log_p: f64) -> i32 {
        unsafe { pocketsphinx_sys::logmath_ln_to_log(self.inner, log_p) }
    }

    /// Convert integer log in base B to natural log (in floating point).
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#abaff7c0ba0976c7cf8f232d8340b7b2f
    pub fn log_to_ln(&self, logb_p: i32) -> f64 {
        unsafe { pocketsphinx_sys::logmath_log_to_ln(self.inner, logb_p) }
    }

    /// Convert base 10 log (in floating point) to integer log in base B.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#abb7027d633f0cabb372504e189e7b4b0
    pub fn log10_to_log(&self, log_p: f64) -> i32 {
        unsafe { pocketsphinx_sys::logmath_log10_to_log(self.inner, log_p) }
    }

    /// Convert base 10 log (in floating point) to float log in base B.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#a9cc2394054dc95d9e94720d036f28a41
    pub fn log10_to_log_float(&self, log_p: f64) -> f32 {
        unsafe { pocketsphinx_sys::logmath_log10_to_log_float(self.inner, log_p) }
    }

    /// Convert integer log in base B to base 10 log (in floating point).
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#ac8fe69a67fcca378b1e5a32428899a23
    pub fn log_to_log10(&self, logb_p: i32) -> f64 {
        unsafe { pocketsphinx_sys::logmath_log_to_log10(self.inner, logb_p) }
    }

    /// Convert float log in base B to base 10 log.
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/structlogmath__t.html#a4e596e6d01e92280e0caa24a48727afb
    pub fn log_float_to_log10(&self, logb_p: f32) -> f64 {
        unsafe { pocketsphinx_sys::logmath_log_float_to_log10(self.inner, logb_p) }
    }

    pub fn get_inner(&self) -> *mut pocketsphinx_sys::logmath_s {
        self.inner
    }
}

impl Drop for LogMath {
    fn drop(&mut self) {
        if !self.retained {
            unsafe { pocketsphinx_sys::logmath_free(self.inner) };
        }
    }
}

pub struct LogMathTableShape {
    /// table size * table width
    pub shape: i32,
    pub size: u32,
    pub width: u32,
    pub shift: u32,
}
