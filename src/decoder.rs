use std::error::Error;

use crate::alignment_iter::Alignment;
use crate::config;
use crate::config::Config;
use crate::fsg::FSG;
use crate::logmath::LogMath;
use crate::nbest_iter::NBestIter;
use crate::search_iter::SearchIter;
use crate::seg_iter::SegIter;

pub struct Decoder {
    inner: *mut pocketsphinx_sys::ps_decoder_t,
    retained: bool,
}

impl Decoder {
    /// Initialize the decoder from a configuration.
    ///
    /// # Arguments
    /// - `config` - Configuration to use for decoder initialization. If `None`, the decoder will be allocated but not initialized. You can proceed to initialize it with `Decoder::reinit()`.
    pub fn new(config: Option<&mut config::Config>) -> Result<Self, Box<dyn Error>> {
        let config_ptr = match config {
            Some(config) => {
                config.set_retained(true);
                config.get_inner()
            }
            None => std::ptr::null_mut(),
        };
        let decoder = unsafe { pocketsphinx_sys::ps_init(config_ptr) };

        if decoder.is_null() {
            Err("Failed to initialize decoder".into())
        } else {
            Ok(Decoder {
                inner: decoder,
                retained: false,
            })
        }
    }

    /// Actives search with the provided name.
    pub fn set_activate_search(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;

        let result = unsafe { pocketsphinx_sys::ps_activate_search(self.inner, c_name.as_ptr()) };

        if result == -1 {
            Err("Failed to activate search".into())
        } else {
            Ok(())
        }
    }

    /// Returns name of current search in decoder
    pub fn current_search(&self) -> Result<String, Box<dyn Error>> {
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

    /// Returns iterator over current searches
    pub fn get_search_iter(&self) -> SearchIter {
        SearchIter::from_decoder(self)
    }

    /// ps_get_lm

    /// ps_add_lm

    /// Adds new search based on N-gram language model.
    ///
    /// Convenient method to load N-gram model and create a search.
    pub fn add_lm_file(&mut self, name: &str, path: &str) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let c_path = std::ffi::CString::new(path)?;

        let result = unsafe {
            pocketsphinx_sys::ps_add_lm_file(self.inner, c_name.as_ptr(), c_path.as_ptr())
        };

        // TODO: Check if this is correct (undocumented...)
        if result == -1 {
            Err("Failed to add LM file".into())
        } else {
            Ok(())
        }
    }

    /// Get the finite-state grammar set object associated with a search.
    ///
    /// # Arguments
    /// - name - Name of the search or `None` for the current search.
    ///
    /// # Returns
    /// The current FSG set object for this decoder, or `None` if name does not correspond to an FSG search.
    pub fn get_fsg(&self, name: Option<&str>) -> Option<FSG> {
        FSG::from_decoder(self, name)
    }

    /// Adds new search based on finite state grammar.
    ///
    /// Associates FSG search with the provided name.
    /// The search can be activated using `Decoder::set_active_search()`.
    pub fn add_fsg(&mut self, name: &str, fsg: &mut FSG) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;

        let result =
            unsafe { pocketsphinx_sys::ps_add_fsg(self.inner, c_name.as_ptr(), fsg.get_inner()) };

        if result == -1 {
            Err("Failed to add FSG".into())
        } else {
            Ok(())
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

    /// Get the keyphrase associated with a KWS search
    ///
    /// # Arguments
    /// - `name` - Name of KWS search, or `None` for current search
    ///
    /// # Returns
    /// The current keyphrase to spot, or `None` if name does not correspond to a KWS search
    pub fn get_kws(&self, name: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        let c_name_ptr = match name {
            Some(name) => std::ffi::CString::new(name)?.as_ptr(),
            None => std::ptr::null(),
        };

        let keyphrase = unsafe { pocketsphinx_sys::ps_get_kws(self.inner, c_name_ptr) };

        if keyphrase.is_null() {
            Ok(None)
        } else {
            let str = unsafe { std::ffi::CStr::from_ptr(keyphrase) }
                .to_str()
                .map_err(|_| "Failed to convert KWS to string")?;

            Ok(Some(str.to_string()))
        }
    }

    /// Adds keyphrases from a file to spotting.
    ///
    /// Associates KWS search with the provided name. The search can be activated using `Decoder::activate_search()`.
    pub fn add_kws_file(&mut self, name: &str, keyfile: &str) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let c_keyfile = std::ffi::CString::new(keyfile)?;

        let result = unsafe {
            pocketsphinx_sys::ps_add_kws(self.inner, c_name.as_ptr(), c_keyfile.as_ptr())
        };

        // TODO: Check if this is correct (undocumented...)
        if result == -1 {
            Err("Failed to add KWS".into())
        } else {
            Ok(())
        }
    }

    /// Adds new keyphrase to spot
    ///
    /// Associates KWS search with the provided name. The search can be activated using `Decoder::activate_search()`.
    pub fn add_keyphrase(&mut self, name: &str, keyphrase: &str) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let c_keyphrase = std::ffi::CString::new(keyphrase)?;

        let result = unsafe {
            pocketsphinx_sys::ps_add_keyphrase(self.inner, c_name.as_ptr(), c_keyphrase.as_ptr())
        };

        // TODO: Check if this is correct (undocumented...)
        if result == -1 {
            Err("Failed to add keyphrase".into())
        } else {
            Ok(())
        }
    }

    // ps_add_allphone

    /// Adds new search based on phone N-gram language model.
    ///
    /// Convenient method to load N-gram model and create a search.
    pub fn add_allphone_file(&mut self, name: &str, path: &str) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let c_path = std::ffi::CString::new(path)?;

        let result = unsafe {
            pocketsphinx_sys::ps_add_allphone_file(self.inner, c_name.as_ptr(), c_path.as_ptr())
        };

        // TODO: Check if this is correct (undocumented...)
        if result == -1 {
            Err("Failed to add allphone file".into())
        } else {
            Ok(())
        }
    }

    /// Set up decoder to force-align a word sequence.
    ///
    /// Unlike the `Decoder::add_*` functions, this activates the search module immediately, since force-alignment is nearly always a single shot.
    /// Currently "under the hood" this is an FSG search but you shouldn't depend on that.
    ///
    /// Decoding proceeds as normal, though only this word sequence will be recognized, with silences and alternate pronunciations inserted.
    /// Word alignments are available with `Decoder::seg_iter()`. To obtain phoneme or state segmentations, you must subsequently call `Decoder::set_alignment()` and re-run decoding.
    /// It's tough son, but it's life.
    pub fn set_align_text(&mut self, words: &str) -> Result<(), Box<dyn Error>> {
        let c_words = std::ffi::CString::new(words)?;

        let result = unsafe { pocketsphinx_sys::ps_set_align_text(self.inner, c_words.as_ptr()) };

        // TODO: Check if this is correct (undocumented...)
        if result == -1 {
            Err("Failed to set align text".into())
        } else {
            Ok(())
        }
    }

    /// Set up decoder to run phone and state-level alignment.
    ///
    /// Unlike the `Decoder::add_*` functions, this activates the search module immediately, since force-alignment is nearly always a single shot.
    ///
    /// To align, run or re-run decoding as usual, then call `Decoder::get_alignment()` to get the resulting alignment.
    /// Note that if you call this function before rerunning decoding, you can obtain the phone and state sequence, but the durations will be invalid (phones and states will inherit the parent word's duration).
    ///
    /// # Returns
    /// `true` if the alignment was successfully set, `false` if an error occured (To align, run or re-run decoding as usual, then call ps_get_alignment() to get the resulting alignment. Note that if you call this function before rerunning decoding, you can obtain the phone and state sequence, but the durations will be invalid (phones and states will inherit the parent word's duration).
    pub fn set_alignment(&mut self, alignment: &Alignment) -> Result<(), Box<dyn Error>> {
        let result =
            unsafe { pocketsphinx_sys::ps_set_alignment(self.inner, alignment.get_inner()) };
        if result == -1 {
            Err("Failed to set alignment".into())
        } else {
            Ok(())
        }
    }

    /// Get the alignment associated with the current search module.
    ///
    /// As noted above, if decoding has not been run, this will contain invalid durations, but that may still be useful if you just want to know the state sequence.
    ///
    /// # Returns
    /// Current alignment or `None`. This pointer is owned by the decoder, so you must call `Alignment::retain()` on it if you wish to keep it outside the lifetime of the decoder.
    pub fn get_alignment(&self) -> Option<Alignment> {
        Alignment::from_decoder(self)
    }

    /// Reinitialize the decoder with updated configuration.
    ///
    /// This function allows you to switch the acoustic model, dictionary, or other configuration without creating an entirely new decoding object.
    ///
    /// Note:
    /// Since the acoustic model will be reloaded, changes made to feature extraction parameters may be overridden if a feat.params file is present.
    /// Any searches created with `Decoder::set_search()` or words added to the dictionary with `Decoder::add_word()` will also be lost. To avoid this you can use `Decoder::reinit_feat()`.
    /// The decoder retains ownership of the pointer config, so you should free it when no longer used.
    pub fn reinit(&mut self, config: &config::Config) -> Result<(), Box<dyn Error>> {
        let result = unsafe { pocketsphinx_sys::ps_reinit(self.inner, config.get_inner()) };

        if result == -1 {
            Err("Failed to reinitialize decoder".into())
        } else {
            Ok(())
        }
    }

    /// Reinitialize only the feature computation with updated configuration.
    ///
    /// This function allows you to switch the feature computation parameters without otherwise affecting the decoder configuration.
    /// For example, if you change the sample rate or the frame rate, and do not want to reconfigure the rest of the decoder.
    ///
    /// Note that if you have set a custom cepstral mean with `Decoder::set_cmn()`, it will be overridden.
    pub fn reinit_feat(&mut self, config: &config::Config) -> Result<(), Box<dyn Error>> {
        let result = unsafe { pocketsphinx_sys::ps_reinit_feat(self.inner, config.get_inner()) };

        if result == -1 {
            Err("Failed to reinitialize decoder".into())
        } else {
            Ok(())
        }
    }

    // ps_get_cmn

    // ps_set_cmn

    /// Returns a retained decoder and assures the underlying pointer is not freed before the returned decoder is dropped.
    ///
    /// This increments the reference count on the decoder, allowing it to be shared between multiple parent objects.
    /// In general you will not need to use this function, ever.
    /// It is mainly here for the convenience of scripting language bindings.
    ///
    /// # Returns
    /// A new `Decoder` object with the retained underlying pointer.
    pub fn retain(&mut self) -> Self {
        let retained_inner = unsafe { pocketsphinx_sys::ps_retain(self.inner) };
        self.retained = true;
        Self {
            inner: retained_inner,
            retained: false,
        }
    }

    /// Get the configuration object for this decoder.
    ///
    /// # Returns
    /// The configuration object for this decoder. The decoder automatically drops this object when it is dropped.
    /// To avoid this, use `Config::retain()`.
    pub fn get_config(&self) -> Config {
        Config::from_decoder(self)
    }

    /// Get the log-math computation object for this decoder.
    ///
    /// # Returns
    /// The log-math object for this decoder.
    /// The decoder owns this log-math. Use `Logmath::retain()` if you wish to reuse it elsewhere.
    pub fn get_logmath(&self) -> LogMath {
        LogMath::from_decoder(self)
    }

    // ps_update_mllr

    /// Reload the pronunciation dictionary from a file.
    /// This function replaces the current pronunciation dictionary with the one stored in the given dictionary. This also causes the active search module(s) to be reinitialized, in the same manner as calling add_word() with update=true.
    ///
    /// # Arguments
    /// - `dictfile`  - Path to dictionary file to load.
    /// - `fdictfile` - Path to filler dictionary to load, or `None` to keep the existing filler dictionary.
    /// - `format`    - Format of the dictionary file, or `None` to determine automatically (currently unused, should be `None`)
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
    pub fn save_dict(&self, dictfile: &str, format: Option<&str>) -> Result<(), Box<dyn Error>> {
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
    /// - `word`   - Word string to add.
    /// - `phones` - Whitespace-separated list of phoneme strings describing pronunciation of `word`.
    /// - `update` - If `true`, update the search module (whichever one is currently active) to recognize the newly added word. If adding multiple words, it is more efficient to pass `false` here in all but the last word.
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
    /// - `word` - Word string to look up.
    ///
    /// # Returns
    /// Whitespace-spearated phone string describing the pronunciation of the word or `None` if word is not present in the dictionary. The string is allocated and must be freed by the user.
    pub fn lookup_word(&self, word: &str) -> Result<Option<String>, Box<dyn Error>> {
        let c_word = std::ffi::CString::new(word)?;

        let c_str = unsafe { pocketsphinx_sys::ps_lookup_word(self.inner, c_word.as_ptr()) };

        if c_str.is_null() {
            Ok(None)
        } else {
            let str = unsafe { std::ffi::CStr::from_ptr(c_str) }.to_str()?;
            Ok(Some(str.to_string()))
        }
    }

    /// Decode a raw audio file.
    ///
    /// No headers are recognized in this files.
    /// The configuration parameters `-samprate` and `-input_endian` are used to determine the sampling rate and endianness of the stream, respectively.
    /// Audio is always assumed to be 16-bit signed PCM.
    ///
    /// # Arguments
    /// - `rawfile`     - Path to the raw audio file.
    /// - `max_samples` - Maximum number of samples to read from rawfh, or `None` to read until end-of-file.
    ///
    /// # Returns
    /// Number of samples of audio.
    pub fn decode_raw_file(
        &mut self,
        rawfile: &str,
        max_samples: Option<i64>,
    ) -> Result<i64, Box<dyn Error>> {
        let c_rawfile = std::ffi::CString::new(rawfile)?;
        let c_file = unsafe { libc::fopen(c_rawfile.as_ptr(), "rb".as_ptr() as *const i8) };
        if c_file.is_null() {
            return Err("Failed to open rawfile".into());
        }
        let c_file_ps = c_file as *mut pocketsphinx_sys::FILE;

        let num_samples = unsafe {
            pocketsphinx_sys::ps_decode_raw(self.inner, c_file_ps, max_samples.unwrap_or(-1))
        };
        unsafe { libc::fclose(c_file) };

        Ok(num_samples)
    }

    /// Decode a senone score dump file.
    ///
    /// # Arguments
    /// - `senscrfile` - Path to the senone score dump file.
    ///
    /// # Returns
    /// Number of frames read.
    pub fn decode_senscr_file(&mut self, senscrfile: &str) -> Result<i32, Box<dyn Error>> {
        let c_senscrfile = std::ffi::CString::new(senscrfile)?;
        let c_file = unsafe { libc::fopen(c_senscrfile.as_ptr(), "rb".as_ptr() as *const i8) };
        if c_file.is_null() {
            return Err("Failed to open senscrfile".into());
        }
        let c_file_ps = c_file as *mut pocketsphinx_sys::FILE;

        let num_frames = unsafe { pocketsphinx_sys::ps_decode_senscr(self.inner, c_file_ps) };
        unsafe { libc::fclose(c_file) };

        Ok(num_frames)
    }

    /// Start processing of the stream of speech.
    #[deprecated(
        since = "0.1.0",
        note = "This function is retained for compatibility, but its only effect is to reset the noise removal statistics, which are otherwise retained across utterances. You do not need to call it."
    )]
    pub fn start_stream(&mut self) -> Result<(), Box<dyn Error>> {
        let result = unsafe { pocketsphinx_sys::ps_start_stream(self.inner) };

        if result == -1 {
            Err("Failed to start stream".into())
        } else {
            Ok(())
        }
    }

    /// Check in-speech status of decoder.
    ///
    /// # Returns
    /// `true` if last buffer contained speech, `false` - otherwise
    #[deprecated(
        since = "0.1.0",
        note = "This function is retained for compatibility but should not be considered a reliable voice activity detector. It will always return `true` between calls to `Decoder::start_utt()` and `Decoder::end_utt()`. You probably want `Endpointer`, but for single frames of data you can also use `VAD`."
    )]
    pub fn get_in_speech(&self) -> Result<bool, Box<dyn Error>> {
        let result = unsafe { pocketsphinx_sys::ps_get_in_speech(self.inner) };

        Ok(result == 1)
    }

    /// Start utterance processing.
    ///
    /// This function should be called before any utterance data is passed to the decoder.
    /// It marks the start of a new utterance and reinitializes internal data structures.
    pub fn start_utt(&mut self) -> Result<(), Box<dyn Error>> {
        let _result = unsafe { pocketsphinx_sys::ps_start_utt(self.inner) };

        Ok(())
    }

    /// Decode raw audio data.
    ///
    /// # Arguments
    /// - `data`      - Raw audio data.
    /// - `no_search` - If `true`, perform feature extraction but don't do any recognition yet.
    ///                 This may be necessary if your processor has trouble doing recognition in real-time.
    /// - `full_utt`  - If `true`, this block of data is a full utterance worth of data. This may allow the recognizer to produce more accurate results.
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

    // ps_process_cep

    /// Get the number of frames of data searched.
    ///
    /// Note that there is a delay between this and the number of frames of audio which have been input to the system.
    /// This is due to the fact that acoustic features are computed using a sliding window of audio, and dynamic features are computed over a sliding window of acoustic features.
    ///
    /// # Returns
    /// Number of frames of speech data which have been recognized so far.
    pub fn get_n_frames(&self) -> i32 {
        unsafe { pocketsphinx_sys::ps_get_n_frames(self.inner) }
    }

    /// End utterance processing.
    pub fn end_utt(&mut self) -> Result<(), Box<dyn Error>> {
        let _result = unsafe { pocketsphinx_sys::ps_end_utt(self.inner) };

        Ok(())
    }

    /// Get hypothesis string and path score.
    ///
    /// # Returns
    /// (hypothesis, score) - Tuple containing the hypothesis string and path score or `None` if no hypothesis is available.
    pub fn get_hyp(&self) -> Result<Option<(String, i32)>, Box<dyn Error>> {
        let mut score = 0;
        let c_str = unsafe { pocketsphinx_sys::ps_get_hyp(self.inner, &mut score) };

        if c_str.is_null() {
            Ok(None)
        } else {
            let str = unsafe { std::ffi::CStr::from_ptr(c_str) }
                .to_str()
                .map_err(|_| "Failed to convert hypothesis to string")?;

            Ok(Some((str.to_string(), score)))
        }
    }

    /// Get posterior probability.
    ///
    /// Note: Unless the -bestpath option is enabled, this function will always return zero (corresponding to a posterior probability of 1.0).
    /// Even if -bestpath is enabled, it will also return zero when called on a partial result.
    /// Ongoing research into effective confidence annotation for partial hypotheses may result in these restrictions being lifted in future versions.
    ///
    /// # Returns
    /// Posterior probability of the best hypothesis.
    pub fn get_prob(&self) -> i32 {
        unsafe { pocketsphinx_sys::ps_get_prob(self.inner) }
    }

    /// ps_get_lattice

    /// Get an iterator over the word segmentation for the best hypothesis.
    ///
    /// # Returns
    /// Iterator over the best hypothesis at this point in decoding. None if no hypothesis is available.
    pub fn get_seg_iter(&self) -> Option<SegIter> {
        SegIter::from_decoder(self)
    }

    /// Get an iterator over the best hypotheses.
    /// The function may return `None` which means that there is no hypothesis available for this utterance.
    pub fn get_nbest_iter(&self) -> Option<NBestIter> {
        NBestIter::from_decoder(self)
    }

    /// Get performance information for the current utterance.
    pub fn get_utt_time(&self) -> DecoderPerformanceInfo {
        let mut speech = 0.0;
        let mut cpu = 0.0;
        let mut wall = 0.0;
        unsafe { pocketsphinx_sys::ps_get_utt_time(self.inner, &mut speech, &mut cpu, &mut wall) };
        DecoderPerformanceInfo { speech, cpu, wall }
    }

    // Get overall performance information.
    pub fn get_all_time(&self) -> DecoderPerformanceInfo {
        let mut speech = 0.0;
        let mut cpu = 0.0;
        let mut wall = 0.0;
        unsafe { pocketsphinx_sys::ps_get_all_time(self.inner, &mut speech, &mut cpu, &mut wall) };
        DecoderPerformanceInfo { speech, cpu, wall }
    }

    pub fn get_inner(&self) -> *mut pocketsphinx_sys::ps_decoder_t {
        self.inner
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        if !self.retained {
            unsafe {
                pocketsphinx_sys::ps_free(self.inner);
            }
        }
    }
}

#[derive(Debug)]
pub struct DecoderPerformanceInfo {
    /// Number of seconds of speech.
    pub speech: f64,
    /// Number of seconds of CPU time used.
    pub cpu: f64,
    /// Number of seconds of wall time used.
    pub wall: f64,
}
