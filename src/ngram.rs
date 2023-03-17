use std::{
    error::Error,
    ffi::{c_char, CStr, CString},
};

use crate::{Config, Decoder, LogMath, NgramIter, NgramSetIter};

pub struct Ngram {
    inner: *mut pocketsphinx_sys::ngram_model_t,
    retained: bool,
}

impl Ngram {
    pub fn from_decoder(decoder: &Decoder, name: Option<&str>) -> Option<Self> {
        let c_name = CString::new(name.unwrap_or("")).unwrap();
        let name_ptr = match name.is_none() {
            true => std::ptr::null(),
            false => c_name.as_ptr(),
        };
        let inner = unsafe { pocketsphinx_sys::ps_get_lm(decoder.get_inner(), name_ptr) };
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                retained: true,
            })
        }
    }

    pub fn from_inner(inner: *mut pocketsphinx_sys::ngram_model_t) -> Self {
        Self {
            inner,
            retained: false,
        }
    }

    /// Read an N-Gram model from a file on disk.
    ///
    /// # Arguments
    /// - `config` - Optional pointer to a set of command-line arguments. Recognized arguments are:
    ///   - `mmap` (boolean) whether to use memory-mapped I/O
    ///   - `lw`   (float32) language weight to apply to the model
    ///   - `wip`  (float32) word insertion penalty to apply to the model
    /// - `file_name` - Path to the file to read.
    /// - `file_type` - Type of the file to read or `NgramFileType::Auto` to determine automatically.
    /// - `logmath` - Log-math parameters to use for probability calculations. Ownership of this object is assumed by the newly created `Ngram`, and you should not attempt to free it manually. If you wish to reuse it elsewhere, you must retain it with `LogMath::retain()`.
    ///
    /// # Returns
    /// A new `Ngram` object or an error.
    pub fn read(
        config: Option<&Config>,
        file_name: &str,
        file_type: NgramFileType,
        logmath: Option<&LogMath>,
    ) -> Result<Self, Box<dyn Error>> {
        let c_file_name = CString::new(file_name).unwrap();
        let inner = unsafe {
            pocketsphinx_sys::ngram_model_read(
                config
                    .map(|c| c.get_inner())
                    .unwrap_or(std::ptr::null_mut()),
                c_file_name.as_ptr(),
                file_type as i32,
                logmath
                    .map(|l| l.get_inner())
                    .unwrap_or(std::ptr::null_mut()),
            )
        };
        if inner.is_null() {
            Err("Failed to read ngram model".into())
        } else {
            Ok(Self {
                inner,
                retained: false,
            })
        }
    }

    /// Write an N-Gram model to disk.
    pub fn write(&self, file_name: &str, file_type: NgramFileType) -> Result<(), Box<dyn Error>> {
        let c_file_name = CString::new(file_name).unwrap();
        let result = unsafe {
            pocketsphinx_sys::ngram_model_write(self.inner, c_file_name.as_ptr(), file_type as i32)
        };
        if result == 0 {
            Ok(())
        } else {
            Err("Failed to write ngram model".into())
        }
    }

    /// Guess the file type for an N-Gram model from the filename.
    ///
    /// # Returns
    /// The file type or `NgramFileType::Invalid` if none could be guessed.
    pub fn file_name_to_type(file_name: &str) -> NgramFileType {
        let c_file_name = CString::new(file_name).unwrap();
        NgramFileType::from_i32(unsafe {
            pocketsphinx_sys::ngram_file_name_to_type(c_file_name.as_ptr())
        })
    }

    /// Get the N-Gram file type from a string.
    ///
    /// # Returns
    /// The file type or `NgramFileType::Invalid` if no such file type exists.
    pub fn str_to_type(str_name: &str) -> NgramFileType {
        let c_str_name = CString::new(str_name).unwrap();
        NgramFileType::from_i32(unsafe { pocketsphinx_sys::ngram_str_to_type(c_str_name.as_ptr()) })
    }

    /// Get the string representation of an N-Gram file type.
    pub fn type_to_str(file_type: NgramFileType) -> &'static str {
        unsafe { CStr::from_ptr(pocketsphinx_sys::ngram_type_to_str(file_type as i32)) }
            .to_str()
            .unwrap()
    }

    /// Retain ownership of an N-Gram model.
    ///
    /// # Returns
    /// A new `Ngram` with the same underlying pointer.
    pub fn retain(&mut self) -> Self {
        let retained_inner = unsafe { pocketsphinx_sys::ngram_model_retain(self.inner) };
        self.retained = true;
        Self {
            inner: retained_inner,
            retained: false,
        }
    }

    /// Case-fold word strings in an N-Gram model.
    /// WARNING: This is not Unicode aware, so any non-ASCII characters will not be converted.
    pub fn casefold(&self, kase: i32) -> i32 {
        unsafe { pocketsphinx_sys::ngram_model_casefold(self.inner, kase) }
    }

    /// Apply a language weight, insertion penalty, and unigram weight to a language model.
    ///
    /// This will change the values output by `Ngram::score()` and friends.
    /// This is done for efficiency since in decoding, these are the only values we actually need.
    /// Call `Ngram::prob()` if you want the "raw" N-Gram probability estimate.
    ///
    /// To remove all weighting, call ngram_apply_weights(model, 1.0, 1.0).
    pub fn apply_weights(&self, lw: f32, wip: f32) -> i32 {
        unsafe { pocketsphinx_sys::ngram_model_apply_weights(self.inner, lw, wip) }
    }

    /// Get the current weights from a language model.
    ///
    /// # Returns
    /// A tuple of `(lw, log_wip)` where `lw` is the language weight and `log_wip` is the logarithm of word insertion penalty.
    pub fn get_weights(&self) -> (f32, i32) {
        let mut log_wip = 0;
        let lw = unsafe { pocketsphinx_sys::ngram_model_get_weights(self.inner, &mut log_wip) };
        (lw, log_wip)
    }

    /// **Note: Untested if this is the correct way to call vararg functions.**
    ///
    /// Get the score (scaled, interpolated log-probability) for a general N-Gram.
    ///
    /// The `words` consist of the history words of the N-Gram, in reverse order.
    ///
    /// ```rust
    /// let score = ngram.score(&["joy", "whole", "a"]);
    /// ```
    ///
    /// This is not the function to use in decoding, because it has some overhead for looking up words. Use `Ngram::ng_score()`, `Ngram::tg_score()`,
    /// or `Ngram::bg_score()` instead. In the future there will probably be a version that takes a general language model state object,
    /// to support suffix-array LM and things like that.
    pub fn score(&self, words: &[&str]) -> i32 {
        let words = words
            .iter()
            .map(|w| CString::new(*w).unwrap())
            .collect::<Vec<_>>();
        let mut c_words = words.iter().map(|w| w.as_ptr()).collect::<Vec<_>>();
        c_words.push(std::ptr::null());
        unsafe { pocketsphinx_sys::ngram_score(self.inner, c_words.as_ptr() as *const _) }
    }

    /// Quick trigram score lookup.
    pub fn tg_score(&self, w3: i32, w2: i32, w1: i32, n_used: &mut i32) -> i32 {
        unsafe { pocketsphinx_sys::ngram_tg_score(self.inner, w3, w2, w1, n_used) }
    }

    /// Quick bigram score lookup.
    pub fn bg_score(&self, w2: i32, w1: i32, n_used: &mut i32) -> i32 {
        unsafe { pocketsphinx_sys::ngram_bg_score(self.inner, w2, w1, n_used) }
    }

    /// Quick general N-Gram score lookup.
    pub fn ng_score(&self, wid: i32, history: &mut [i32], n_used: &mut i32) -> i32 {
        unsafe {
            pocketsphinx_sys::ngram_ng_score(
                self.inner,
                wid,
                history.as_mut_ptr(),
                history.len() as i32,
                n_used,
            )
        }
    }

    /// **Note: Untested if this is the correct way to call vararg functions.**
    ///
    /// Get the "raw" log-probability for a general N-Gram.
    ///
    /// This returns the log-probability of an N-Gram, as defined in the language model file, before any language weighting,
    /// interpolation, or insertion penalty has been applied.
    ///
    /// Note: When backing off to a unigram from a bigram or trigram, the unigram weight (interpolation with uniform) is not removed.
    pub fn probv(&self, words: &[&str]) -> i32 {
        let words = words
            .iter()
            .map(|w| CString::new(*w).unwrap())
            .collect::<Vec<_>>();
        let mut c_words = words.iter().map(|w| w.as_ptr()).collect::<Vec<_>>();
        c_words.push(std::ptr::null());
        unsafe { pocketsphinx_sys::ngram_probv(self.inner, c_words.as_ptr() as *const _) }
    }

    /// Get the "raw" log-probability for a general N-Gram.
    ///
    /// This returns the log-probability of an N-Gram, as defined in the language model file,
    /// before any language weighting, interpolation, or insertion penalty has been applied.
    ///
    /// Note: When backing off to a unigram from a bigram or trigram, the unigram weight (interpolation with uniform) is not removed.
    pub fn prob(&self, words: &[&str]) -> i32 {
        let c_words: Vec<CString> = words
            .iter()
            .map(|word| CString::new(*word).unwrap())
            .collect();
        let c_words_ptrs: Vec<*const c_char> = c_words.iter().map(|word| word.as_ptr()).collect();
        unsafe {
            pocketsphinx_sys::ngram_prob(self.inner, c_words_ptrs.as_ptr(), words.len() as i32)
        }
    }

    /// Quick "raw" probability lookup for a general N-Gram.
    ///
    /// See documentation for `Ngram::ng_score()` and `Ngram::apply_weights()` for an explanation of this.
    pub fn ng_prob(&self, wid: i32, history: &mut [i32], n_used: &mut i32) -> i32 {
        unsafe {
            pocketsphinx_sys::ngram_ng_prob(
                self.inner,
                wid,
                history.as_mut_ptr(),
                history.len() as i32,
                n_used,
            )
        }
    }

    /// Convert score to "raw" log-probability.
    ///
    /// Note: The unigram weight (interpolation with uniform) is not removed, since there is no way to know which order of N-Gram generated score.
    ///
    /// # Arguments
    /// * `score` - The score to convert.
    ///
    /// # Returns
    /// The raw log-probability value.
    pub fn score_to_prob(&self, score: i32) -> i32 {
        unsafe { pocketsphinx_sys::ngram_score_to_prob(self.inner, score) }
    }

    /// Look up numerical word ID.
    pub fn wid(&self, word: &str) -> i32 {
        let c_word = CString::new(word).unwrap();
        unsafe { pocketsphinx_sys::ngram_wid(self.inner, c_word.as_ptr()) }
    }

    /// Look up word string for numerical word ID.
    pub fn word(&self, wid: i32) -> String {
        let c_word = unsafe { pocketsphinx_sys::ngram_word(self.inner, wid) };
        let c_word = unsafe { CStr::from_ptr(c_word) };
        c_word.to_string_lossy().into_owned()
    }

    /// Get the unknown word ID for a language model.
    ///
    /// Language models can be either "open vocabulary" or "closed vocabulary".
    /// The difference is that the former assigns a fixed non-zero unigram probability to unknown words,
    /// while the latter does not allow unknown words (or, equivalently, it assigns them zero probability).
    /// If this is a closed vocabulary model, this function will return NGRAM_INVALID_WID.
    ///
    /// # Returns
    /// The ID for the unknown word, or NGRAM_INVALID_WID if none exists.
    pub fn unknown_wid(&self) -> i32 {
        unsafe { pocketsphinx_sys::ngram_unknown_wid(self.inner) }
    }

    /// Get the "zero" log-probability value for a language model.
    pub fn zero(&self) -> i32 {
        unsafe { pocketsphinx_sys::ngram_zero(self.inner) }
    }

    /// Get the order of the N-gram model (i.e. the "N" in "N-gram")
    pub fn get_size(&self) -> i32 {
        unsafe { pocketsphinx_sys::ngram_model_get_size(self.inner) }
    }

    /// Get the counts of the various N-grams in the model.
    pub fn get_counts(&self) -> Vec<u32> {
        let counts = unsafe { pocketsphinx_sys::ngram_model_get_counts(self.inner) };
        let mut counts_vec = Vec::new();
        for i in 0..self.get_size() as usize {
            counts_vec.push(unsafe { *counts.offset(i as isize) });
        }
        counts_vec
    }

    /// Iterate over all M-grams.
    ///
    /// # Arguments
    /// - `m` - Order of the M-Grams requested minus one (i.e. order of the history)
    ///
    /// # Returns
    /// An iterator over the requested M, or `None` if no N-grams of order M+1 exist.
    pub fn mgrams(&self, m: i32) -> Option<NgramIter> {
        let inner = unsafe { pocketsphinx_sys::ngram_model_mgrams(self.inner, m) };
        if inner.is_null() {
            None
        } else {
            Some(NgramIter::from_inner(inner))
        }
    }

    /// Get an iterator over M-grams pointing to the specified M-gram.
    pub fn iter(&self, words: &[&str]) -> NgramIter {
        let words = words
            .iter()
            .map(|w| CString::new(*w).unwrap())
            .collect::<Vec<_>>();
        let mut c_words = words.iter().map(|w| w.as_ptr()).collect::<Vec<_>>();
        c_words.push(std::ptr::null());
        let inner =
            unsafe { pocketsphinx_sys::ngram_iter(self.inner, c_words.as_ptr() as *const _) };
        NgramIter::from_inner(inner)
    }

    /// Get an iterator over M-grams pointing to the specified M-gram.
    pub fn ng_iter(&self, wid: i32, history: &mut [i32]) -> NgramIter {
        let inner = unsafe {
            pocketsphinx_sys::ngram_ng_iter(
                self.inner,
                wid,
                history.as_mut_ptr(),
                history.len() as i32,
            )
        };
        NgramIter::from_inner(inner)
    }

    /// Add a word (unigram) to the language model.
    ///
    /// Note: The semantics of this are not particularly well-defined for model sets, and may be subject to change.
    /// Currently this will add the word to all of the submodels
    ///
    /// # Arguments
    /// * `word` - Text of the word to add.
    /// * `weight` - Weight of this word relative to the uniform distribution.
    ///
    /// # Returns
    /// The word ID for the new word.
    pub fn add_word(&self, word: &str, weight: f32) -> i32 {
        let c_word = CString::new(word).unwrap();
        unsafe { pocketsphinx_sys::ngram_model_add_word(self.inner, c_word.as_ptr(), weight) }
    }

    /// Read a class definition file and add classes to a language model.
    ///
    /// This function assumes that the class tags have already been defined as unigrams in the language model.
    /// All words in the class definition will be added to the vocabulary as special in-class words.
    /// For this reason is is necessary that they not have the same names as any words in the general unigram distribution.
    /// The convention is to suffix them with ":class_tag", where class_tag is the class tag minus the enclosing square brackets.
    pub fn read_classdef(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let c_file_name = CString::new(file_name).unwrap();
        let result = unsafe {
            pocketsphinx_sys::ngram_model_read_classdef(self.inner, c_file_name.as_ptr())
        };
        if result == 0 {
            Ok(())
        } else {
            Err("Failed to read classdef".into())
        }
    }

    /// Add a new class to a language model.
    ///
    /// If `classname` already exists in the unigram set for model, then it will be converted to a class tag, and `classweight` will be ignored.
    /// Otherwise, a new unigram will be created as in `Ngram::add_word()`.
    pub fn add_class(
        &self,
        classname: &str,
        classweight: f32,
        words: &[&str],
        weights: &[f32],
    ) -> i32 {
        let classname = CString::new(classname).unwrap();
        let words = words
            .iter()
            .map(|word| CString::new(*word).unwrap())
            .collect::<Vec<_>>();
        let mut words = words
            .iter()
            .map(|word| word.as_ptr() as *mut i8)
            .collect::<Vec<_>>();
        unsafe {
            pocketsphinx_sys::ngram_model_add_class(
                self.inner,
                classname.as_ptr(),
                classweight,
                words.as_mut_ptr(),
                weights.as_ptr(),
                words.len() as i32,
            )
        }
    }

    /// Add a word to a class in a language model.
    ///
    /// # Arguments
    /// - `classname` - Name of the class to add this word to.
    /// - `word` - Text of the word to add.
    /// - `weight` - Weight of this word relative to the within-class uniform distribution.
    ///
    /// # Returns
    /// The word ID for the new word.
    pub fn add_class_word(&self, classname: &str, word: &str, weight: f32) -> i32 {
        let classname = CString::new(classname).unwrap();
        let word = CString::new(word).unwrap();
        unsafe {
            pocketsphinx_sys::ngram_model_add_class_word(
                self.inner,
                classname.as_ptr(),
                word.as_ptr(),
                weight,
            )
        }
    }

    /// Create a set of language models sharing a common space of word IDs.
    ///
    /// This function creates a meta-language model which groups together a set of language models, synchronizing word IDs between them.
    /// To use this language model, you can either select a submodel to use exclusively using `Ngram::set_select()`, or interpolate between scores from all models.
    /// To do the latter, you can either pass `Some(weights)` value of the weights parameter, or re-activate interpolation later on by calling `Ngram::set_interp()`.
    ///
    /// In order to make this efficient, there are some restrictions on the models that can be grouped together.
    /// The most important (and currently the only) one is that they must all share the same log-math parameters.
    ///
    /// # Arguments
    /// - `config` - Any configuration parameters to be shared between models.
    /// - `models` - Array of `Ngram` to previously created language models.
    /// - `names` - Array of strings to use as unique identifiers for LMs.
    /// - `weights` - Array of weights to use in interpolating LMs, or `None` for no interpolation.
    pub fn set_init(
        config: &Config,
        models: &[Ngram],
        names: &[&str],
        weights: Option<&[f32]>,
    ) -> Ngram {
        let mut models = models.iter().map(|m| m.inner).collect::<Vec<_>>();
        let c_names = names
            .iter()
            .map(|s| CString::new(*s).unwrap())
            .collect::<Vec<_>>();
        let mut c_names = c_names
            .iter()
            .map(|s| s.as_ptr() as *mut i8)
            .collect::<Vec<_>>();
        let weights_ptr = match weights {
            Some(weights) => weights.as_ptr(),
            None => std::ptr::null(),
        };
        let inner = unsafe {
            pocketsphinx_sys::ngram_model_set_init(
                config.get_inner(),
                models.as_mut_ptr(),
                c_names.as_mut_ptr(),
                weights_ptr,
                models.len() as i32,
            )
        };
        Ngram {
            inner,
            retained: false,
        }
    }

    /// Read a set of language models from a control file.
    ///
    /// This file creates a language model set from a "control file" of the type used in Sphinx-II and Sphinx-III. File format (optional stuff is indicated by enclosing in []):
    ///
    /// ```
    /// [{ LMClassFileName LMClassFilename ... }]
    /// TrigramLMFileName LMName [{ LMClassName LMClassName ... }]
    /// TrigramLMFileName LMName [{ LMClassName LMClassName ... }]
    /// ...
    /// (There should be whitespace around the { and } delimiters.)
    /// ```
    ///
    /// This is an extension of the older format that had only TrigramLMFilenName and LMName pairs. The new format allows a set of LMClass files to be read in and referred to by the trigram LMs.
    ///
    /// No "comments" allowed in this file.
    ///
    /// # Arguments
    /// - `config` - Configuration parameters.
    /// - `lmctlfile` - Path to the language model control file.
    /// - `logmath` - Log-math parameters to use for probability calculations. Ownership of this object is assumed by the newly created `Ngram`. If you wish to reuse it elsewhere, you must retain it with `Logmath::retain()`.
    pub fn set_read(config: &Config, lmctlfile: &str, logmath: &LogMath) -> Ngram {
        let c_lmctlfile = CString::new(lmctlfile).unwrap();
        let inner = unsafe {
            pocketsphinx_sys::ngram_model_set_read(
                config.get_inner(),
                c_lmctlfile.as_ptr(),
                logmath.get_inner(),
            )
        };
        Ngram {
            inner,
            retained: false,
        }
    }

    /// Returns the number of language models in a set.
    pub fn set_count(&self) -> i32 {
        unsafe { pocketsphinx_sys::ngram_model_set_count(self.inner) }
    }

    /// Begin iterating over language models in a set.
    ///
    /// # Returns
    /// Iterator pointing to the first language model, or `None` if no models remain.
    pub fn set_iter(&self) -> Option<NgramSetIter> {
        let inner = unsafe { pocketsphinx_sys::ngram_model_set_iter(self.inner) };

        if inner.is_null() {
            None
        } else {
            Some(NgramSetIter::from_inner(inner))
        }
    }

    /// Select a single language model from a set for scoring.
    ///
    /// # Returns
    /// The newly selected language model, or `None` if no language model by that name exists.
    pub fn set_select(&self, name: &str) -> Option<Ngram> {
        let c_name = CString::new(name).unwrap();
        let inner =
            unsafe { pocketsphinx_sys::ngram_model_set_select(self.inner, c_name.as_ptr()) };
        if inner.is_null() {
            None
        } else {
            Some(Ngram {
                inner,
                retained: false,
            })
        }
    }

    /// Look up a language model by name from a set.
    ///
    /// # Returns
    /// Language model corresponding to name, or `None` if no language model by that name exists.
    pub fn set_lookup(&self, name: &str) -> Option<Ngram> {
        let c_name = CString::new(name).unwrap();
        let inner =
            unsafe { pocketsphinx_sys::ngram_model_set_lookup(self.inner, c_name.as_ptr()) };
        if inner.is_null() {
            None
        } else {
            Some(Ngram {
                inner,
                retained: false,
            })
        }
    }

    /// Get the current language model name, if any.
    pub fn set_current(&self) -> String {
        let c_name = unsafe { pocketsphinx_sys::ngram_model_set_current(self.inner) };
        let c_name = unsafe { CStr::from_ptr(c_name) };
        c_name.to_string_lossy().into_owned()
    }

    /// Set interpolation weights for a set and enables interpolation.
    ///
    /// If weights is `None`, any previously initialized set of weights will be used.
    /// If no weights were specified to `Ngram::set_init()`, then a uniform distribution will be used.
    pub fn set_interp(&self, names: &[&str], weights: Option<&[f32]>) -> Ngram {
        let c_names: Vec<_> = names.iter().map(|s| CString::new(*s).unwrap()).collect();
        let mut c_names: Vec<_> = c_names.iter().map(|s| s.as_ptr()).collect();
        let weights_ptr = match weights {
            Some(weights) => weights.as_ptr(),
            None => std::ptr::null(),
        };
        let inner = unsafe {
            pocketsphinx_sys::ngram_model_set_interp(self.inner, c_names.as_mut_ptr(), weights_ptr)
        };
        Ngram {
            inner,
            retained: false,
        }
    }

    /// Add a language model to a set.
    ///
    /// # Arguments
    /// - `model` - Language model to add.
    /// - `name` - The name to associate with this model.
    /// - `weight` - Interpolation weight for this model, relative to the uniform distribution. `1.0` is a safe value.
    /// - `reuse_widmap` - Reuse the existing word-ID mapping in set. Any new words present in model will not be added to the word-ID mapping in this case.
    pub fn set_add(&self, model: &Ngram, name: &str, weight: f32, reuse_widmap: bool) -> Ngram {
        let c_name = CString::new(name).unwrap();
        let inner = unsafe {
            pocketsphinx_sys::ngram_model_set_add(
                self.inner,
                model.inner,
                c_name.as_ptr(),
                weight,
                reuse_widmap as i32,
            )
        };
        Ngram {
            inner,
            retained: false,
        }
    }

    /// Remove a language model from a set.
    ///
    /// # Arguments
    /// - `name` - The name associated with the model to remove.
    /// - `reuse_widmap` - Reuse the existing word-ID mapping in set.
    pub fn set_remove(&self, name: &str, reuse_widmap: bool) -> Ngram {
        let c_name = CString::new(name).unwrap();
        let inner = unsafe {
            pocketsphinx_sys::ngram_model_set_remove(
                self.inner,
                c_name.as_ptr(),
                reuse_widmap as i32,
            )
        };
        Ngram {
            inner,
            retained: false,
        }
    }

    /// Set the word-to-ID mapping for this model set.
    pub fn set_map_words(&self, words: &[&str]) {
        let c_words: Vec<_> = words.iter().map(|s| CString::new(*s).unwrap()).collect();
        let mut c_words: Vec<_> = c_words.iter().map(|s| s.as_ptr()).collect();
        unsafe {
            pocketsphinx_sys::ngram_model_set_map_words(
                self.inner,
                c_words.as_mut_ptr(),
                words.len() as i32,
            );
        }
    }

    /// Query the word-ID mapping for the current language model.
    ///
    /// # Returns
    /// The local word ID in the current language model, or NGRAM_INVALID_WID if set_wid is invalid or interpolation is enabled.
    pub fn set_current_wid(&self, set_wid: i32) -> i32 {
        unsafe { pocketsphinx_sys::ngram_model_set_current_wid(self.inner, set_wid) }
    }

    /// Test whether a word ID corresponds to a known word in the current state of the language model set.
    ///
    /// # Returns
    /// If there is a current language model, returns non-zero if set_wid corresponds to a known word in that language model.
    /// Otherwise, returns non-zero if `set_wid` corresponds to a known word in any language model.
    pub fn set_known_wid(&self, set_wid: i32) -> i32 {
        unsafe { pocketsphinx_sys::ngram_model_set_known_wid(self.inner, set_wid) }
    }

    /// Flush any cached N-Gram information
    pub fn flush(&self) {
        unsafe { pocketsphinx_sys::ngram_model_flush(self.inner) }
    }

    pub fn get_inner(&self) -> *mut pocketsphinx_sys::ngram_model_t {
        self.inner
    }
}

impl Drop for Ngram {
    fn drop(&mut self) {
        if !self.retained {
            unsafe {
                pocketsphinx_sys::ngram_model_free(self.inner);
            }
        }
    }
}

pub enum NgramFileType {
    Invalid = -1,
    Auto = 0,
    Arpa = 1,
    Bin = 2,
}

impl NgramFileType {
    pub fn from_i32(value: i32) -> Self {
        match value {
            -1 => NgramFileType::Invalid,
            0 => NgramFileType::Auto,
            1 => NgramFileType::Arpa,
            2 => NgramFileType::Bin,
            _ => panic!("Invalid NgramFileType value"),
        }
    }
}
