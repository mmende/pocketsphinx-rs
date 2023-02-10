use std::error::Error;

use crate::decoder::Decoder;

pub struct Config {
    inner: *mut pocketsphinx_sys::ps_config_t,
    retained: bool,
}

impl Config {
    /// Create a configuration with default values.
    ///
    /// # Returns
    /// Newly created configuration or an Error on failure (should not happen, but check it anyway).
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let config = unsafe { pocketsphinx_sys::ps_config_init(std::ptr::null_mut()) };

        if config.is_null() {
            Err("Failed to initialize config".into())
        } else {
            Ok(Config {
                inner: config,
                retained: false,
            })
        }
    }

    /// Create a configuration with default values.
    ///
    /// This is a convenience function that calls `new` and `default_search_args`.
    ///
    /// # Returns
    /// Newly created configuration or an Error on failure (should not happen, but check it anyway).
    pub fn default() -> Result<Self, Box<dyn Error>> {
        let mut config = Self::new()?;
        config.default_search_args();
        Ok(config)
    }

    /// Get config from decoder.
    pub fn from_decoder(decoder: &Decoder) -> Self {
        let inner = unsafe { pocketsphinx_sys::ps_get_config(decoder.get_inner()) };
        Self {
            inner,
            retained: true,
        }
    }

    /// Returns a retained config and assures the underlying config is not freed before the retained config is dropped.
    pub fn retain(&mut self) -> Self {
        let retained_inner = unsafe { pocketsphinx_sys::ps_config_retain(self.inner) };
        self.retained = true;
        Config {
            inner: retained_inner,
            retained: false,
        }
    }

    /// Create a decoder with this configuration.
    pub fn init_decoder(&mut self) -> Result<Decoder, Box<dyn Error>> {
        Decoder::new(Some(self))
    }

    /// Create a configuration by parsing slightly extended JSON.
    ///
    /// This function parses a JSON object in non-strict mode to produce a `Config`.
    /// Configuration parameters are given without a leading dash, and do not need to be quoted, nor does the object need to be enclosed in curly braces, nor are commas necessary between key/value pairs.
    /// Basically, it's degenerate YAML. So, for example, this is accepted:
    ///
    /// ```yaml
    /// hmm: fr-fr
    /// samprate: 8000
    /// keyphrase: "hello world"
    /// ```
    ///
    /// Of course, valid JSON is also accepted, but who wants to use that.
    /// Well, mostly. Unicode escape sequences (e.g. "\u0020") are not supported at the moment, so please don't use them.
    ///
    /// # Returns
    /// Newly created configuration or an Error on failure (such as invalid or missing parameters).
    pub fn from_json(json: &str) -> Result<Self, Box<dyn Error>> {
        let c_json = std::ffi::CString::new(json)?;
        let config = unsafe {
            pocketsphinx_sys::ps_config_parse_json(std::ptr::null_mut(), c_json.as_ptr())
        };

        if config.is_null() {
            Err("Failed to initialize config".into())
        } else {
            Ok(Config {
                inner: config,
                retained: false,
            })
        }
    }

    /// Updates the configuration by parsing slightly extended JSON.
    ///
    /// This function parses a JSON object in non-strict mode to produce a `Config`.
    /// Configuration parameters are given without a leading dash, and do not need to be quoted, nor does the object need to be enclosed in curly braces, nor are commas necessary between key/value pairs.
    /// Basically, it's degenerate YAML. So, for example, this is accepted:
    ///
    /// ```yaml
    /// hmm: fr-fr
    /// samprate: 8000
    /// keyphrase: "hello world"
    /// ```
    ///
    /// Of course, valid JSON is also accepted, but who wants to use that.
    /// Well, mostly. Unicode escape sequences (e.g. "\u0020") are not supported at the moment, so please don't use them.
    ///
    /// # Returns
    /// Ok or an Error on failure (such as invalid or missing parameters).
    pub fn extend_from_json(&mut self, json: &str) -> Result<(), Box<dyn Error>> {
        let c_json = std::ffi::CString::new(json)?;
        let result = unsafe { pocketsphinx_sys::ps_config_parse_json(self.inner, c_json.as_ptr()) };

        if result.is_null() {
            Err("Failed to extend config".into())
        } else {
            self.inner = result;
            Ok(())
        }
    }

    /// Construct JSON from a configuration object.
    ///
    /// Unlike Config::from_json or Config::extend_from_json, this actually produces valid JSON ;-)
    pub fn serialize_json(&self) -> Result<String, Box<dyn Error>> {
        let c_json = unsafe { pocketsphinx_sys::ps_config_serialize_json(self.inner) };

        if c_json.is_null() {
            Err("Failed to serialize config".into())
        } else {
            let json = unsafe { std::ffi::CStr::from_ptr(c_json) }
                .to_str()
                .map_err(|_| "Failed to convert config to string")?
                .to_string();
            Ok(json)
        }
    }

    /// Get the type of a parameter and if the parameter is required.
    /// # Returns
    /// A tuple of the parameter type and a boolean indicating whether the parameter is required.
    pub fn typeof_param(&self, name: &str) -> Result<(ParamType, bool), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let param_type = unsafe { pocketsphinx_sys::ps_config_typeof(self.inner, c_name.as_ptr()) };
        if param_type == 0 {
            return Err("Unknown parameter".into());
        }
        // Required = 1<<0
        // Integer = 1<<1
        // Float = 1<<2
        // String = 1<<3
        // Boolean = 1<<4
        let required = (param_type & 1) == 1;
        let param_type = match param_type & 0b11110 {
            0b10 => ParamType::Integer,
            0b100 => ParamType::Float,
            0b1000 => ParamType::String,
            0b10000 => ParamType::Boolean,
            _ => {
                return Err("Unknown parameter type".into());
            }
        };
        Ok((param_type, required))
    }

    /// Validate configuration.
    ///
    /// Currently this just checks that you haven't specified multiple types of grammars or language models at the same time.
    pub fn is_valid(&self) -> bool {
        let validation_result = unsafe { pocketsphinx_sys::ps_config_validate(self.inner) };
        validation_result == 0
    }

    /// Get a boolean-valued parameter.
    ///
    /// If the parameter does not have an integer or boolean type, this will print an error and return an Err.
    pub fn get_bool(&self, name: &str) -> Result<bool, Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let value = unsafe { pocketsphinx_sys::ps_config_bool(self.inner, c_name.as_ptr()) };

        if value == -1 {
            Err("Failed to get config value".into())
        } else {
            Ok(value == 1)
        }
    }

    /// Set a boolean-valued parameter.
    ///
    /// If the parameter does not have an integer or boolean type, this will convert `value` appropriately.
    pub fn set_bool(&mut self, name: &str, value: bool) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let value = if value { 1 } else { 0 };

        let _result =
            unsafe { pocketsphinx_sys::ps_config_set_bool(self.inner, c_name.as_ptr(), value) };

        Ok(())
    }

    /// Get an integer-valued parameter.
    ///
    /// If the parameter does not have an integer or boolean type, this will print an error and return 0. So don't do that.
    pub fn get_int(&self, name: &str) -> Result<i64, Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let value = unsafe { pocketsphinx_sys::ps_config_int(self.inner, c_name.as_ptr()) };

        Ok(value)
    }

    /// Set an integer-valued parameter.
    ///
    /// If the parameter does not have an integer or boolean type, this will convert `value` appropriately.
    pub fn set_int(&mut self, name: &str, value: i64) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;

        let _result =
            unsafe { pocketsphinx_sys::ps_config_set_int(self.inner, c_name.as_ptr(), value) };

        Ok(())
    }

    /// Get a floating-point parameter.
    ///
    /// If the parameter does not have a floating-point type, this will print an error and return 0.
    pub fn get_float(&self, name: &str) -> Result<f64, Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let value = unsafe { pocketsphinx_sys::ps_config_float(self.inner, c_name.as_ptr()) };

        Ok(value)
    }

    /// Set a floating-point parameter.
    ///
    /// If the parameter does not have a floating-point type, this will convert `value` appropriately.
    pub fn set_float(&mut self, name: &str, value: f64) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;

        let _result =
            unsafe { pocketsphinx_sys::ps_config_set_float(self.inner, c_name.as_ptr(), value) };

        Ok(())
    }

    /// Get a string parameter.
    ///
    /// If the parameter does not have a string type, this will print an error and return an Err.
    /// Notably, it will NOT format an `integer` or `float` for you, because that would involve allocating memory. So don't do that.
    pub fn get_str(&self, name: &str) -> Result<String, Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let value = unsafe { pocketsphinx_sys::ps_config_str(self.inner, c_name.as_ptr()) };

        if value.is_null() {
            Err("Failed to get config value".into())
        } else {
            Ok(unsafe { std::ffi::CStr::from_ptr(value) }
                .to_str()
                .unwrap()
                .to_string())
        }
    }

    /// Set a string-valued parameter.
    ///
    /// If the parameter does not have a string type, this will convert `value` appropriately.
    /// For boolean parameters, any string matching /^[yt1]/ will be `true`, while any string matching /^[nf0]/ will be `false`.
    pub fn set_str(&mut self, name: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let c_value = std::ffi::CString::new(value)?;

        let _result = unsafe {
            pocketsphinx_sys::ps_config_set_str(self.inner, c_name.as_ptr(), c_value.as_ptr())
        };

        Ok(())
    }

    /// Set configuration parameters (actually just sample rate) from a sound file.
    ///
    /// If the file is unreadable, unsupported or incompatible with the existing feature extraction parameters, this will print an error message and fail.
    ///
    /// If it is of an unknown type, it will be treated as raw data.
    /// So beware! Currently we only support WAV and NIST Sphere files.
    /// We attempt to recognize Ogg, MP3 (but not really, because it is very difficult to do reliably), and FLAC, but do not support them.
    /// For everything else, there's SoX (tm).
    ///
    /// Currently, the file must be seekable, so you can't use this on standard input, for instance.
    ///
    pub fn from_soundfile(
        &mut self,
        soundfile: &str,
        name: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let c_soundfile = std::ffi::CString::new(soundfile)?;
        let c_name_ptr = match name {
            Some(name) => std::ffi::CString::new(name)?.as_ptr(),
            None => std::ptr::null_mut(),
        };
        // Create C File pointer
        let c_file = unsafe { libc::fopen(c_soundfile.as_ptr(), "rb".as_ptr() as *const i8) };
        if c_file.is_null() {
            return Err("Failed to open soundfile".into());
        }
        let c_file_ps = c_file as *mut pocketsphinx_sys::FILE;
        let result =
            unsafe { pocketsphinx_sys::ps_config_soundfile(self.inner, c_file_ps, c_name_ptr) };
        unsafe { libc::fclose(c_file) };
        if result == -1 {
            Err("Failed to configure from soundfile".into())
        } else {
            Ok(())
        }
    }

    /// Read a WAV header and set configuration parameters.
    ///
    /// This works like `Config::from_soundfile()` but assumes that you already know it's a WAV file.
    ///
    /// Unlike Config::from_soundfile(), the file does _not_ have to be seekable.
    pub fn from_wavfile(
        &mut self,
        wavfile: &str,
        name: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let c_soundfile = std::ffi::CString::new(wavfile)?;
        let c_name_ptr = match name {
            Some(name) => std::ffi::CString::new(name)?.as_ptr(),
            None => std::ptr::null_mut(),
        };
        // Create C File pointer
        let c_file = unsafe { libc::fopen(c_soundfile.as_ptr(), "rb".as_ptr() as *const i8) };
        if c_file.is_null() {
            return Err("Failed to open wavfile".into());
        }
        let c_file_ps = c_file as *mut pocketsphinx_sys::FILE;
        let result =
            unsafe { pocketsphinx_sys::ps_config_wavfile(self.inner, c_file_ps, c_name_ptr) };
        unsafe { libc::fclose(c_file) };
        if result == -1 {
            Err("Failed to configure from wavfile".into())
        } else {
            Ok(())
        }
    }

    /// Read a NIST header and set configuration parameters.
    ///
    /// This works like `Config::from_soundfile()` but assumes that you already know it's a NIST file.
    ///
    /// Unlike Config::from_soundfile(), the file does _not_ have to be seekable.
    pub fn from_nistfile(
        &mut self,
        nistfile: &str,
        name: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let c_soundfile = std::ffi::CString::new(nistfile)?;
        let c_name_ptr = match name {
            Some(name) => std::ffi::CString::new(name)?.as_ptr(),
            None => std::ptr::null_mut(),
        };
        // Create C File pointer
        let c_file = unsafe { libc::fopen(c_soundfile.as_ptr(), "rb".as_ptr() as *const i8) };
        if c_file.is_null() {
            return Err("Failed to open nistfile".into());
        }
        let c_file_ps = c_file as *mut pocketsphinx_sys::FILE;
        let result =
            unsafe { pocketsphinx_sys::ps_config_nistfile(self.inner, c_file_ps, c_name_ptr) };
        unsafe { libc::fclose(c_file) };
        if result == -1 {
            Err("Failed to configure from nistfile".into())
        } else {
            Ok(())
        }
    }

    /// Sets default file paths and parameters based on configuration.
    pub fn expand_model_config(&mut self) {
        unsafe {
            pocketsphinx_sys::ps_expand_model_config(self.inner);
        }
    }

    /// Sets default acoustic and language model if they are not set explicitly.
    ///
    /// This function fills in the configuration with the default acoustic and language models and dictionary, if (and this is a badly implemented heuristic) they do not seem to be already filled in.
    /// It is preferable for you to call this before doing any other configuration to avoid confusion.
    ///
    /// The default models are looked for in the directory returned by default_modeldir(), or, if the `POCKETSPHINX_PATH` environment variable is set, this function will look there instead.
    ///
    /// If no global model directory was defined at compilation time (this is useful for relocatable installs such as the Python module) and `POCKETSPHINX_PATH` is not set, this will simply do nothing.
    pub fn default_search_args(&mut self) {
        unsafe {
            pocketsphinx_sys::ps_default_search_args(self.inner);
        }
    }

    pub fn get_inner(&self) -> *mut pocketsphinx_sys::ps_config_t {
        self.inner
    }

    /// Used internally to check specify if the underlying pointer should be freed or is owned by another object.
    pub fn set_retained(&mut self, retained: bool) {
        self.retained = retained;
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        if !self.retained {
            unsafe {
                pocketsphinx_sys::ps_config_free(self.inner);
            }
        }
    }
}

pub enum ParamType {
    Integer,
    Boolean,
    Float,
    String,
}
