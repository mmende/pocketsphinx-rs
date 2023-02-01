use std::error::Error;

use crate::config;

pub struct PsDecoder {
    inner: *mut pocketsphinx_sys::ps_decoder_t,
}

impl PsDecoder {
    pub fn new(config: &config::PsConfig) -> Result<Self, Box<dyn Error>> {
        let decoder = unsafe { pocketsphinx_sys::ps_init(config.get_inner()) };

        if decoder.is_null() {
            Err("Failed to initialize decoder".into())
        } else {
            Ok(PsDecoder { inner: decoder })
        }
    }

    /// Start utterance processing.
    ///
    /// This function should be called before any utterance data is passed to the decoder. It marks the start of a new utterance and reinitializes internal data structures.
    pub fn start_utt(&mut self) -> Result<(), Box<dyn Error>> {
        let _result = unsafe { pocketsphinx_sys::ps_start_utt(self.inner) };

        Ok(())
    }

    /// End utterance processing.
    pub fn end_utt(&mut self) -> Result<(), Box<dyn Error>> {
        let _result = unsafe { pocketsphinx_sys::ps_end_utt(self.inner) };

        Ok(())
    }

    /// Decode raw audio data.
    ///
    /// # Arguments
    /// - data - Raw audio data.
    /// - no_search - If `true`, perform feature extraction but don't do any recognition yet. This may be necessary if your processor has trouble doing recognition in real-time.
    /// - full_utt - If `true`, this block of data is a full utterance worth of data. This may allow the recognizer to produce more accurate results.
    ///
    /// # Returns
    /// Number of frames of data searched.
    pub fn process_raw(
        &mut self,
        data: &[i16],
        no_search: bool,
        full_utt: bool,
    ) -> Result<i32, Box<dyn Error>> {
        let result = unsafe {
            pocketsphinx_sys::ps_process_raw(
                self.inner,
                data.as_ptr(),
                data.len() as usize,
                no_search as i32,
                full_utt as i32,
            )
        };

        if result == -1 {
            Err("Failed to process raw data".into())
        } else {
            Ok(result)
        }
    }

    /// Get hypothesis string and path score.
    ///
    /// # Returns
    /// (hypothesis, score) - Tuple containing the hypothesis string and path score.
    pub fn get_hyp(&mut self) -> Result<(String, i32), Box<dyn Error>> {
        let mut score = 0;
        let c_str = unsafe { pocketsphinx_sys::ps_get_hyp(self.inner, &mut score) };

        if c_str.is_null() {
            Err("Failed to get hypothesis".into())
        } else {
            let str = unsafe { std::ffi::CStr::from_ptr(c_str) }
                .to_str()
                .map_err(|_| "Failed to convert hypothesis to string")?;

            Ok((str.to_string(), score))
        }
    }

    /// Adds new search using JSGF model.
    ///
    /// # Arguments
    /// - name - Name of the search.
    /// - path - Path to JSGF model.
    pub fn add_jsgf_file(&mut self, name: &str, path: &str) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let c_path = std::ffi::CString::new(path)?;

        let result = unsafe {
            pocketsphinx_sys::ps_add_jsgf_file(self.inner, c_name.as_ptr(), c_path.as_ptr())
        };

        if result == -1 {
            Err("Failed to add JSGF file".into())
        } else {
            Ok(())
        }
    }

    /// Adds new search using JSGF model.
    ///
    /// Convenience method to parse JSGF model from string and create a search.
    ///
    /// # Arguments
    /// - name - Name of the search.
    /// - jsgf - JSGF model.
    pub fn add_jsgf_string(&mut self, name: &str, jsgf: &str) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let c_jsgf = std::ffi::CString::new(jsgf)?;

        let result = unsafe {
            pocketsphinx_sys::ps_add_jsgf_string(self.inner, c_name.as_ptr(), c_jsgf.as_ptr())
        };

        if result == -1 {
            Err("Failed to add JSGF string".into())
        } else {
            Ok(())
        }
    }

    /// Actives search with the provided name.
    pub fn activate_search(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;

        let result = unsafe { pocketsphinx_sys::ps_activate_search(self.inner, c_name.as_ptr()) };

        if result == -1 {
            Err("Failed to activate search".into())
        } else {
            Ok(())
        }
    }

    /// Returns name of current search in decoder
    pub fn current_search(&mut self) -> Result<String, Box<dyn Error>> {
        let c_str = unsafe { pocketsphinx_sys::ps_current_search(self.inner) };

        if c_str.is_null() {
            Err("Failed to get current search".into())
        } else {
            let str = unsafe { std::ffi::CStr::from_ptr(c_str) }
                .to_str()
                .map_err(|_| "Failed to convert current search to string")?;

            Ok(str.to_string())
        }
    }

    /// Removes a search module and releases its resources.
    ///
    /// Removes a search module previously added with using `add_jsgf()`, `add_fsg()`, `add_lm()`, `add_kws()`, etc.
    pub fn remove_search(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;

        let result = unsafe { pocketsphinx_sys::ps_remove_search(self.inner, c_name.as_ptr()) };

        if result == -1 {
            Err("Failed to remove search".into())
        } else {
            Ok(())
        }
    }

    /// Reload the pronunciation dictionary from a file.
    /// This function replaces the current pronunciation dictionary with the one stored in the given dictionary. This also causes the active search module(s) to be reinitialized, in the same manner as calling add_word() with update=true.
    ///
    /// # Arguments
    /// - dictfile - Path to dictionary file to load.
    /// - fdictfile - Path to filler dictionary to load, or `None` to keep the existing filler dictionary.
    /// - format - Format of the dictionary file, or `None` to determine automatically (currently unused, should be `None`)
    pub fn load_dict(
        &mut self,
        dictfile: &str,
        fdictfile: Option<&str>,
        format: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let c_dictfile = std::ffi::CString::new(dictfile)?;

        let c_fdictfile_ptr = if fdictfile.is_none() {
            std::ptr::null()
        } else {
            let c_fdictfile = std::ffi::CString::new(fdictfile.unwrap())?;
            c_fdictfile.as_ptr()
        };

        let c_format_ptr = if format.is_none() {
            std::ptr::null()
        } else {
            let c_format = std::ffi::CString::new(format.unwrap())?;
            c_format.as_ptr()
        };

        let result = unsafe {
            pocketsphinx_sys::ps_load_dict(
                self.inner,
                c_dictfile.as_ptr(),
                c_fdictfile_ptr,
                c_format_ptr,
            )
        };

        if result == -1 {
            Err("Failed to load dictionary".into())
        } else {
            Ok(())
        }
    }

    /// Dump the current pronunciation dictionary to a file.
    ///
    /// This function dumps the current pronunciation dictionary to a text file.
    pub fn save_dict(
        &mut self,
        dictfile: &str,
        format: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let c_dictfile = std::ffi::CString::new(dictfile)?;

        let c_format_ptr = if format.is_none() {
            std::ptr::null()
        } else {
            let c_format = std::ffi::CString::new(format.unwrap())?;
            c_format.as_ptr()
        };

        let result = unsafe {
            pocketsphinx_sys::ps_save_dict(self.inner, c_dictfile.as_ptr(), c_format_ptr)
        };

        if result == -1 {
            Err("Failed to save dictionary".into())
        } else {
            Ok(())
        }
    }

    /// Add a word to the pronunciation dictionary.
    ///
    /// This function adds a word to the pronunciation dictionary and the current language model (but, obviously, not to the current FSG if FSG mode is enabled). If the word is already present in one or the other, it does whatever is necessary to ensure that the word can be recognized.
    ///
    /// # Arguments
    /// - word - Word string to add.
    /// - phones - Whitespace-separated list of phoneme strings describing pronunciation of `word`.
    /// - update - If `true`, update the search module (whichever one is currently active) to recognize the newly added word. If adding multiple words, it is more efficient to pass `false` here in all but the last word.
    pub fn add_word(
        &mut self,
        word: &str,
        phones: &str,
        update: bool,
    ) -> Result<(), Box<dyn Error>> {
        let c_word = std::ffi::CString::new(word)?;
        let c_phones = std::ffi::CString::new(phones)?;

        let result = unsafe {
            pocketsphinx_sys::ps_add_word(
                self.inner,
                c_word.as_ptr(),
                c_phones.as_ptr(),
                update as i32,
            )
        };

        if result == -1 {
            Err("Failed to add word".into())
        } else {
            Ok(())
        }
    }

    /// Look up a word in the dictionary and return phone transcription for it.
    ///
    /// # Arguments
    /// - word - Word string to look up.
    ///
    /// # Returns
    /// Whitespace-spearated phone string describing the pronunciation of the word or `None` if word is not present in the dictionary. The string is allocated and must be freed by the user.
    pub fn lookup_word(&mut self, word: &str) -> Result<Option<String>, Box<dyn Error>> {
        let c_word = std::ffi::CString::new(word)?;

        let c_str = unsafe { pocketsphinx_sys::ps_lookup_word(self.inner, c_word.as_ptr()) };

        if c_str.is_null() {
            Ok(None)
        } else {
            let str = unsafe { std::ffi::CStr::from_ptr(c_str) }.to_str()?;
            Ok(Some(str.to_string()))
        }
    }
}

impl Drop for PsDecoder {
    fn drop(&mut self) {
        unsafe {
            pocketsphinx_sys::ps_free(self.inner);
        }
    }
}
