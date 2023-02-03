use pocketsphinx::{config::Config, decoder::Decoder};

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

    let mut config = Config::new().expect("Failed to create config");
    config
        .set_str("hmm", format!("{}/en-us/en-us", model_dir).as_str())
        .expect("Failed to set hmm");
    config
        .set_str("lm", format!("{}/en-us/en-us.lm.bin", model_dir).as_str())
        .expect("Failed to set lm");
    config
        .set_str(
            "dict",
            format!("{}/en-us/cmudict-en-us.dict", model_dir).as_str(),
        )
        .expect("Failed to set dict");

    let mut decoder = Decoder::new(Some(&mut config)).expect("Failed to create decoder");
    let jsgf_path = format!("{}/examples/numbers.jsgf", manifest_dir);
    decoder
        .add_jsgf_file("numbers", jsgf_path.as_str())
        .expect("Failed to add JSGF string");
    decoder
        .activate_search("numbers")
        .expect("Failed to activate search");
    decoder.start_utt().unwrap();
    decoder.process_raw(&audio_i16, false, false).unwrap();
    decoder.end_utt().unwrap();

    let (hyp, _score) = decoder.get_hyp().unwrap();
    println!("Hypothesis: {}", hyp);
}
