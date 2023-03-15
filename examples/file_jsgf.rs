// This example shows how to use pocketsphinx-rs to decode audio from a wav file
// using the english default model with a JSGF grammar that recognizes single digit numbers.
// To run this example, place a 16-bit, 16kHz, mono wav file named "audio.wav" in
// the examples/data directory and run it with `cargo run --example file_jsgf`.

use pocketsphinx::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let audio_path = format!("{}/examples/data/audio.wav", manifest_dir);
    let audio = std::fs::read(audio_path)?;

    // Skip the header and convert to i16
    let audio_i16: Vec<i16> = audio[44..]
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    let model_dir = format!("{}/sys/pocketsphinx/model", manifest_dir);

    let hmm = format!("{}/en-us/en-us", model_dir);
    let dict = format!("{}/en-us/cmudict-en-us.dict", model_dir);

    // Create a config and set the acoustic model, dictionary, and language model manually (instead of calling `Config::default()`)
    let mut config = Config::new()?;
    config.set_str("hmm", hmm.as_str())?;
    config.set_str("dict", dict.as_str())?;

    // To use the default language model instead of a JSGF grammar, uncomment the following lines and comment out the JSGF grammar lines:
    // let lm = format!("{}/en-us/en-us.lm.bin", model_dir);
    // config.set_str("lm", lm.as_str())?;

    // Initialize a decoder
    let mut decoder = config.init_decoder()?;

    // Use JSGF grammar
    let jsgf_path = format!("{}/examples/data/numbers.jsgf", manifest_dir);
    decoder.add_jsgf_file("numbers", jsgf_path.as_str())?;
    decoder.set_activate_search("numbers")?;

    // Decode audio
    decoder.start_utt()?;
    decoder.process_raw(&audio_i16, false, false)?;
    decoder.end_utt()?;

    let hyp_result = decoder.get_hyp()?;
    if let Some((hyp, _score)) = hyp_result {
        println!("Hypothesis: {}", hyp);
    } else {
        println!("No hypothesis");
    }

    Ok(())
}
