// This example shows how to use pocketsphinx-rs to decode audio from a wav file
// using the english default model with a JSGF grammar that recognizes single digit numbers.
// To run this example, place a 16-bit, 16kHz, mono wav file named "audio.wav" in
// the examples directory and run it with `cargo run --example file_jsgf`.

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

    let model_dir = format!("{}/sys/pocketsphinx/model", manifest_dir);

    let hmm = format!("{}/en-us/en-us", model_dir);
    let dict = format!("{}/en-us/cmudict-en-us.dict", model_dir);

    // Create a config
    let mut config = Config::new().expect("Failed to create config");
    config
        .set_str("hmm", hmm.as_str())
        .expect("Failed to set hmm");
    config
        .set_str("dict", dict.as_str())
        .expect("Failed to set dict");

    // To use the default language model instead of a JSGF grammar, uncomment the following lines and comment out the JSGF grammar lines:
    // let lm = format!("{}/en-us/en-us.lm.bin", model_dir);
    // config.set_str("lm", lm.as_str()).expect("Failed to set lm");

    // Initialize a decoder
    let mut decoder = config.init_decoder().expect("Failed to create decoder");

    // Use JSGF grammar
    let jsgf_path = format!("{}/examples/numbers.jsgf", manifest_dir);
    decoder
        .add_jsgf_file("numbers", jsgf_path.as_str())
        .expect("Failed to add JSGF string");
    decoder
        .activate_search("numbers")
        .expect("Failed to activate search");

    // Decode audio
    decoder.start_utt().unwrap();
    decoder.process_raw(&audio_i16, false, false).unwrap();
    decoder.end_utt().unwrap();

    let (hyp, _score) = decoder.get_hyp().unwrap();
    println!("Hypothesis: {}", hyp);
}
