// This example shows how to use pocketsphinx-rs to use the segmentation iterator to get word level infos.
// To run this example, place a 16-bit, 16kHz, mono wav file named "audio.wav" in
// the examples directory and run it with `cargo run --example segments`.

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
    let mut decoder = config.init_decoder()?;

    // Decode audio
    decoder.start_utt()?;
    decoder.process_raw(&audio_i16, false, false)?;
    decoder.end_utt()?;

    for seg in decoder.get_seg_iter().unwrap() {
        let word = seg.get_word();
        let frames = seg.get_frames();
        let start_s = frames.start as f32 / 100.0;
        let end_s = frames.end as f32 / 100.0;
        println!("[{} - {}]\t{}", start_s, end_s, word);
    }

    // We can also get segments for nbest hypotheses
    // println!();
    // println!("N-Best (3): ");
    // for nbest in decoder.get_nbest_iter().unwrap().take(3) {
    //     for seg in nbest.get_seg() {
    //         let word = seg.get_word();
    //         let frames = seg.get_frames();
    //         let start_s = frames.start as f32 / 100.0;
    //         let end_s = frames.end as f32 / 100.0;
    //         println!("[{} - {}]\t{}", start_s, end_s, word);
    //     }
    //     println!();
    // }

    Ok(())
}
