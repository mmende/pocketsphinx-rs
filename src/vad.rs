use std::error::Error;

use crate::endpointer::Endpointer;

pub struct VAD {
    inner: *mut pocketsphinx_sys::ps_vad_t,
    retained: bool,
}

impl VAD {
    /// Initialize voice activity detection.
    ///
    /// # Arguments
    /// - `mode`            - "Aggressiveness" of voice activity detection. Stricter values are less likely to misclassify non-speech as speech.
    /// - `sample_rate`     - Sampling rate of input, or `None` for default (which can be obtained with `VAD::get_sample_rate()`). Only `8000`, `16000`, `32000`, `48000` are directly supported.
    ///                       See `VAD::set_input_params()` for more information.
    /// - `frame_length`    - Frame length in seconds, or `None` for the default. Only `0.01`, `0.02`, `0.03` currently supported. Actual value may differ, you must use `VAD::get_frame_length()` to obtain it.
    pub fn new(
        mode: VADMode,
        sample_rate: Option<i32>,
        frame_length: Option<f64>,
    ) -> Result<Self, Box<dyn Error>> {
        let sample_rate = sample_rate.unwrap_or_else(|| 0);
        let frame_length = frame_length.unwrap_or_else(|| 0.0);
        let inner =
            unsafe { pocketsphinx_sys::ps_vad_init(mode as u32, sample_rate, frame_length) };
        if inner.is_null() {
            Err("Failed to initialize VAD".into())
        } else {
            Ok(Self {
                inner,
                retained: false,
            })
        }
    }

    /// Initialize voice activity detection from an endpointer.
    pub fn from_endpointer(endpointer: &Endpointer) -> Self {
        let vad = unsafe { pocketsphinx_sys::ps_endpointer_vad(endpointer.get_inner()) };
        Self {
            inner: vad,
            retained: true,
        }
    }

    /// Returns a retained VAD and assures that the underlying pointer is not freed before the returned VAD is dropped.
    ///
    /// # Returns
    /// Voice activity detector with incremented reference count.
    pub fn retain(&mut self) -> Self {
        let retained_inner = unsafe { pocketsphinx_sys::ps_vad_retain(self.inner) };
        self.retained = true;
        Self {
            inner: retained_inner,
            retained: false,
        }
    }

    /// Set the input parameters for voice activity detection.
    ///
    /// # Arguments
    /// - `sample_rate`     - Sampling rate of input, or `None` for default (which can be obtained with `VAD::get_sample_rate()`). Only `8000`, `16000`, `32000`, `48000` are directly supported, others will use the closest supported rate (within reason).
    ///                       Note that this means that the actual frame length may not be exactly the one requested, so you must always use the one returned by `VAD::get_frame_size()` (in samples) or `VAD::get_frame_length()` (in seconds).
    /// - `frame_length`    - Requested frame length in seconds, or `None` for the default. Only `0.01`, `0.02`, `0.03` currently supported.
    ///                       Actual frame length may be different, you must always use `VAD::get_frame_length()` to obtain it.
    pub fn set_input_params(
        &mut self,
        sample_rate: Option<i32>,
        frame_length: Option<f64>,
    ) -> Result<(), Box<dyn Error>> {
        let sample_rate = sample_rate.unwrap_or_else(|| 0);
        let frame_length = frame_length.unwrap_or_else(|| 0.0);
        let result = unsafe {
            pocketsphinx_sys::ps_vad_set_input_params(self.inner, sample_rate, frame_length)
        };
        if result == 0 {
            Ok(())
        } else {
            Err("Failed to set VAD input parameters".into())
        }
    }

    /// Get the sampling rate expected by voice activity detection.
    ///
    /// # Returns
    /// Expected sampling rate.
    pub fn get_sample_rate(&self) -> i32 {
        let sample_rate = unsafe { pocketsphinx_sys::ps_vad_sample_rate(self.inner) };
        sample_rate
    }

    /// Get the number of samples expected by voice activity detection.
    ///
    /// You **must** always ensure that the buffers passed to `VAD::classify()` contain this number of samples (zero-pad them if necessary).
    ///
    /// # Returns
    /// Size, in samples, of the frames passed to `VAD::classify()`.
    pub fn get_frame_size(&self) -> usize {
        let frame_size = unsafe { pocketsphinx_sys::ps_vad_frame_size(self.inner) };
        frame_size
    }

    /// Classify a frame as speech or not speech.
    pub fn classify(&mut self, frame: &[i16]) -> VADClass {
        let result = unsafe { pocketsphinx_sys::ps_vad_classify(self.inner, frame.as_ptr()) };
        match result {
            -1 => VADClass::Error,
            0 => VADClass::NotSpeech,
            1 => VADClass::Speech,
            _ => unreachable!(),
        }
    }

    /// Default sampling rate for voice activity detector.
    ///
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/vad_8h.html#a619d5a74e526164718dfee5ed9a48202
    pub fn get_default_sample_rate() -> i32 {
        16_000
    }

    /// Default frame length for voice activity detector.
    ///
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/vad_8h.html#a42e1e50b03fb55c0b0377cf017c70390
    pub fn get_default_frame_length() -> f64 {
        0.03
    }

    /// Get the _actual_ length of a frame in seconds.
    ///
    /// This may differ from the value requested in `VAD::set_input_params()`.
    ///
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/vad_8h.html#a8605289aba98c9ef20a53d77fd1bcfe4
    pub fn get_frame_length(&self) -> i32 {
        self.get_frame_size() as i32 / self.get_sample_rate()
    }
}

impl Drop for VAD {
    fn drop(&mut self) {
        if !self.retained {
            unsafe { pocketsphinx_sys::ps_vad_free(self.inner) };
        }
    }
}

pub enum VADMode {
    Loose = 0,
    MediumLoose = 1,
    MediumStrict = 2,
    Strict = 3,
}

pub enum VADClass {
    Error = -1,
    NotSpeech = 0,
    Speech = 1,
}
