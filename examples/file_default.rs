// This example shows how to use pocketsphinx-rs to decode audio from a wav file
// using the default model.
// To run this example, place a 16-bit, 16kHz, mono wav file named "audio.wav" in
// the examples directory and run it with `cargo run --example file_default`.

use pocketsphinx::config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let audio_path = format!("{}/examples/audio.wav", manifest_dir);
    let audio = std::fs::read(audio_path)?;

    // Skip the header and convert to i16
    let audio_i16: Vec<i16> = audio[44..]
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    // Create a config and set default acoustic model, dictionary, and language model
    let mut config = Config::default()?;

    // Initialize a decoder
    let mut decoder = config.init_decoder().expect("Failed to create decoder");

    // Decode audio
    decoder.start_utt()?;
    decoder.process_raw(&audio_i16, false, false)?;
    decoder.end_utt()?;

    let (hyp, _score) = decoder.get_hyp()?;
    println!("Hypothesis: {}", hyp);
    Ok(())
}
