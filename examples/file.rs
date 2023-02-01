use pocketsphinx::{config::PsConfig, decoder::PsDecoder};

fn main() {
    let audio = std::fs::read("audio.wav").unwrap();
    // Skip the header and convert to i16
    let audio_i16: Vec<i16> = audio[44..]
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    let model_dir = "../sys/pocketsphinx/model";

    let mut config = PsConfig::new().expect("Failed to create config");
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

    let mut decoder = PsDecoder::new(&config).expect("Failed to create decoder");
    decoder
        .add_jsgf_file("numbers", "numbers.jsgf")
        .expect("Failed to add JSGF string");
    decoder
        .activate_search("numbers")
        .expect("Failed to activate search");
    decoder.start_utt().unwrap();
    decoder.process_raw(&audio_i16, false, false).unwrap();
    decoder.end_utt().unwrap();

    let (hyp, score) = decoder.get_hyp().unwrap();
    println!("{}, {}", hyp, score);
}
