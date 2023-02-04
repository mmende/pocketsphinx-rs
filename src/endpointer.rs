use std::error::Error;

use crate::vad::{VADMode, VAD};

pub struct Endpointer {
    inner: *mut pocketsphinx_sys::ps_endpointer_t,
}

impl Endpointer {
    /// Initialize endpointing.
    ///
    /// # Arguments
    /// - `window` - Seconds of audio to use in speech start/end decision, or `None` to use the default (`Endpointer::default_window()`).
    /// - `ratio` - Ratio of frames needed to trigger start/end decision, or `None` for the default (`Endpointer::default_ratio()`).
    /// - `mode` - "Aggressiveness" of voice activity detection. Stricter values (see `VADMode`) are less likely to misclassify non-speech as speech.
    /// - `sample_rate` - Sampling rate of input, or `None` for default (which can be obtained with `VAD::sample_rate()`). Only `8000`, `16000`, `32000`, `48000` are directly supported, others will use the closest supported rate (within reason).
    ///                   Note that this means that the actual frame length may not be exactly the one requested, so you must always use the one returned by `Endpointer::frame_size()` (in samples) or `Endpointer::frame_length()` (in seconds).
    /// - `frame_length` - Requested frame length in seconds, or `None` for the default. Only `0.01`, `0.02`, `0.03` currently supported.
    ///                    **Actual frame length may be different, you must always use `Endpointer::frame_length()` to obtain it.**
    pub fn new(
        window: Option<f64>,
        ratio: Option<f64>,
        mode: VADMode,
        sample_rate: Option<i32>,
        frame_length: Option<f64>,
    ) -> Result<Self, Box<dyn Error>> {
        let window = window.unwrap_or(0.0);
        let ratio = ratio.unwrap_or(0.0);
        let sample_rate = sample_rate.unwrap_or(0);
        let frame_length = frame_length.unwrap_or(0.0);
        let inner = unsafe {
            pocketsphinx_sys::ps_endpointer_init(
                window,
                ratio,
                mode as u32,
                sample_rate,
                frame_length,
            )
        };
        if inner.is_null() {
            Err("Failed to initialize endpointer".into())
        } else {
            Ok(Self { inner })
        }
    }

    /// Initialize endpointing with default parameters.
    pub fn default() -> Result<Self, Box<dyn Error>> {
        Self::new(None, None, VADMode::Loose, None, None)
    }

    /// Retain a pointer to endpointer
    ///
    /// # Returns
    /// Endpointer with incremented reference count.
    pub fn retain(&self) {
        unsafe { pocketsphinx_sys::ps_endpointer_retain(self.inner) };
    }

    /// Get the voice activity detector used by the endpointer.
    ///
    /// # Returns
    /// `VAD`. The endpointer retains ownership of this object, so you must use `VAD::retain()` if you wish to use it outside of the lifetime of the endpointer.
    pub fn vad(&self) -> VAD {
        let mut vad = VAD::from_endpointer(self);
        vad.set_owned_by_endpointer(true);
        vad
    }

    /// Process a frame of audio, returning a frame if in a speech region.
    ///
    /// Note that the endpointer is not thread-safe. You must call all endpointer functions from the same thread.
    ///
    /// # Arguments
    /// - `frame` - Frame of audio. Must be the same length as the frame length specified when the endpointer was created.
    ///
    /// # Returns
    /// `None` if no speech available, or a slice of a frame of `Endpointer::frame_size()` samples (no more and no less).
    pub fn process(&mut self, frame: &[i16]) -> Option<&[i16]> {
        let result = unsafe { pocketsphinx_sys::ps_endpointer_process(self.inner, frame.as_ptr()) };
        if result.is_null() {
            None
        } else {
            let frame = unsafe { std::slice::from_raw_parts(result, self.frame_size()) };
            Some(frame)
        }
    }

    /// Process remaining samples at end of stream.
    ///
    /// Note that the endpointer is not thread-safe. You must call all endpointer functions from the same thread.
    ///
    /// # Arguments
    /// - `frame` - Frame of data, must contain `Endpointer::frame_size()` samples or less.
    ///
    /// # Returns
    /// Slice to available samples, or `None` if none available.
    pub fn end_stream(&mut self, frame: &[i16]) -> Option<&[i16]> {
        let mut out_nsamp = 0;
        let result = unsafe {
            pocketsphinx_sys::ps_endpointer_end_stream(
                self.inner,
                frame.as_ptr(),
                frame.len(),
                &mut out_nsamp,
            )
        };
        if result.is_null() {
            None
        } else {
            let available_samples =
                unsafe { std::slice::from_raw_parts(result, self.frame_size()) };
            Some(available_samples)
        }
    }

    /// Get the current state (speech/not-speech) of the endpointer.
    ///
    /// This function can be used to detect speech/non-speech transitions.
    /// If it returns `true`, and a subsequent call to `Endpointer::process()` returns `Some`, this indicates a transition to speech.
    /// Conversely, if `Endpointer::process()` returns `Some` and a subsequent call to this function returns `false`, this indicates a transition to non-speech.
    ///
    /// # Returns
    /// `true` if in a speech segment after processing the last frame of data.
    pub fn get_in_speech(&self) -> bool {
        unsafe { pocketsphinx_sys::ps_endpointer_in_speech(self.inner) != 0 }
    }

    /// Get the start time of the last speech segment.
    pub fn get_speech_start(&self) -> f64 {
        unsafe { pocketsphinx_sys::ps_endpointer_speech_start(self.inner) }
    }

    /// Get the end time of the last speech segment.
    pub fn get_speech_end(&self) -> f64 {
        unsafe { pocketsphinx_sys::ps_endpointer_speech_end(self.inner) }
    }

    /// Default window in seconds of audio to use for speech start/end decision.
    ///
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/endpointer_8h.html#a66481b47838efb4704b9483893cd1c8a
    pub fn default_window() -> f64 {
        pocketsphinx_sys::PS_ENDPOINTER_DEFAULT_WINDOW
    }

    /// Default ratio of frames in window to trigger start/end decision.
    ///
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/endpointer_8h.html#a98c9cba29a99e4ef2d08865da20c1d3c
    pub fn default_ratio() -> f64 {
        pocketsphinx_sys::PS_ENDPOINTER_DEFAULT_RATIO
    }

    /// Get the frame size required by the endpointer.
    ///
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/endpointer_8h.html#aaa16760235cea4c0a06db70c907bc576
    pub fn frame_size(&self) -> usize {
        self.vad().frame_size()
    }

    /// Get the frame length required by the endpointer.
    ///
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/endpointer_8h.html#a65e580fb57829e172863093b8d8bfdf7
    pub fn frame_length(&self) -> i32 {
        self.vad().frame_length()
    }

    /// Get the sample rate required by the endpointer.
    ///
    /// @see https://cmusphinx.github.io/doc/pocketsphinx/endpointer_8h.html#a1b52b6d6bf58004f463ad01697b1076f
    pub fn sample_rate(&self) -> i32 {
        self.vad().get_sample_rate()
    }

    pub fn get_inner(&self) -> *mut pocketsphinx_sys::ps_endpointer_t {
        self.inner
    }
}

impl Drop for Endpointer {
    fn drop(&mut self) {
        unsafe { pocketsphinx_sys::ps_endpointer_free(self.inner) };
    }
}
