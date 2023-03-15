// This example shows how to use pocketsphinx-rs to decode live microphone audio using the cpal crate for audio input
// and the dasp crate for resampling from the samplerate provided by the input device.
// The example uses the english default model with a keyphrase spotter and a JSGF grammar (commands.jsgf) that gets activated after the keyphrase `oh mighty computer` was detected.
// To run this example: `cargo run --example live`.

use std::sync::mpsc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dasp::{interpolate::linear::Linear, signal, Signal};
use pocketsphinx::{Config, Endpointer};

#[derive(PartialEq)]
enum SearchMode {
    KeywordSpotter,
    Grammar,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a channel to be able to send audio data to the decoder
    let (tx, rx) = mpsc::channel();

    // Create a microphone input stream
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("Failed to get default input device");
    let stream_config = device
        .default_input_config()
        .expect("Failed to get default input config");
    let sample_rate = stream_config.sample_rate().0 as f64;
    let stream_config = stream_config.into();

    println!(
        "Using input device \"{}\" at samplerate {}",
        device.name()?,
        sample_rate
    );

    // Create the audio stream
    let stream = device.build_input_stream(
        &stream_config,
        move |data: &[f32], _| {
            // Resample the chunk to 16 bit, 16 khz using dasp
            let data_f32 = data.to_vec();
            let mut signal = signal::from_interleaved_samples_iter::<_, [f32; 1]>(data_f32);
            let a = signal.next();
            let b = signal.next();
            let interpolator = Linear::new(a, b);
            let converter = signal.from_hz_to_hz(interpolator, sample_rate, 16_000.0 as f64);
            let mut chunk_i16 = Vec::new();
            for sample in converter.until_exhausted() {
                chunk_i16.push((sample[0] * i16::MAX as f32) as i16);
            }
            // Send the chunk to the decoder
            tx.send(chunk_i16).unwrap();
        },
        |_| {
            panic!("An error occured");
        },
        None,
    )?;
    // Start the stream
    stream.play()?;
    println!("Listening... (press Ctrl+C to exit)");

    // Start the decoder thread
    let mut config = Config::default()?;
    let mut decoder = config.init_decoder()?;
    let ep = Endpointer::default()?;

    // We add two searches, a keyword spotter and a grammar after the keyword spotter has been detected
    decoder.add_keyphrase("keyword", "oh mighty computer")?;
    decoder.add_jsgf_file(
        "commands",
        format!(
            "{}/examples/data/commands.jsgf",
            std::env::var("CARGO_MANIFEST_DIR")?
        )
        .as_str(),
    )?;
    decoder.set_activate_search("keyword")?;
    let mut search_mode = SearchMode::KeywordSpotter;

    // Endpointer requires a fixed frame size
    // In order to always process frame size parts of the audio stream
    // we extend a frame_cache with the chunks we receive from the stream
    // and always process frame size parts
    let frame_size = ep.get_frame_size();
    let mut frame_cache = Vec::with_capacity(frame_size * 2);

    for chunk in rx {
        // Extend the cache with the chunk
        frame_cache.extend(chunk);
        // If the chunk is smaller than the frame size add it to the cache
        if frame_cache.len() < frame_size {
            continue;
        }
        // Get frame size chunks from the chunk
        let mut unprocessed_offset = frame_cache.len();
        let frames = frame_cache.chunks_exact(frame_size);
        unprocessed_offset -= frames.remainder().len();
        for frame in frames {
            // Check if the endpointer had detected speech in the previous chunk (must be done before calling process)
            let prev_in_speech = ep.get_in_speech();

            // Check if the endpointer detected speech in the current frame
            let process_result = ep.process(frame);
            if process_result.is_some() {
                // If speech was not detected in the previous frame, start utterance
                if !prev_in_speech {
                    let speech_start = ep.get_speech_start();
                    println!("Speech started at {}", speech_start);
                    decoder.start_utt()?;
                }
                // Process the speech_frame returned by the process method
                let speech_frame = process_result.unwrap();
                decoder.process_raw(speech_frame, false, false)?;
                // Check if the decoder has a hypothesis
                match decoder.get_hyp()? {
                    Some((hyp, _score)) => {
                        if search_mode == SearchMode::Grammar {
                            println!("Partial hypothesis: {}", hyp);
                        }
                    }
                    None => {}
                }
                // Check if speech has ended
                if !ep.get_in_speech() {
                    let speech_end = ep.get_speech_end();
                    println!("Speech ended at {}", speech_end);
                    decoder.end_utt()?;
                    // Check if the decoder has a hypothesis
                    match decoder.get_hyp()? {
                        Some((hyp, _score)) => {
                            if search_mode == SearchMode::KeywordSpotter {
                                if hyp == "oh mighty computer" {
                                    println!("Keyphrase detected, switching to grammar search");
                                    // Switch to grammar search
                                    decoder.set_activate_search("commands")?;
                                    search_mode = SearchMode::Grammar;
                                }
                            } else {
                                println!("Hypothesis: {}", hyp);
                                // Switch back to keyword spotter search
                                decoder.set_activate_search("keyword")?;
                                search_mode = SearchMode::KeywordSpotter;
                            }
                        }
                        None => {}
                    };
                }
            }
        }
        // Remove the processed frames from the cache
        frame_cache.drain(..unprocessed_offset);
    }

    Ok(())
}
