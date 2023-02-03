// This example shows how to use pocketsphinx-rs to decode audio from a wav file
// using the default model.
// To run this example, place a 16-bit, 16kHz, mono wav file named "audio.wav" in
// the examples directory and run it with `cargo run --example file_default`.

use pocketsphinx::config::Config;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let audio_path = format!("{}/examples/audio.wav", manifest_dir);
    let audio = std::fs::read(audio_path).unwrap();
    // Skip the header and convert to i16
    let audio_i16: Vec<i16> = audio[44..]
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    // Create a config and set default acoustic model, dictionary, and language model
    let mut config = Config::new().expect("Failed to create config");
    config.default_search_args();

    // Initialize a decoder
    let mut decoder = config.init_decoder().expect("Failed to create decoder");

    // Decode audio
    decoder.start_utt().unwrap();
    decoder.process_raw(&audio_i16, false, false).unwrap();
    decoder.end_utt().unwrap();

    let (hyp, _score) = decoder.get_hyp().unwrap();
    println!("Hypothesis: {}", hyp);
}
