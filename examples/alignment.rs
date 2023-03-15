// This example shows how to use pocketsphinx-rs to force align text and get word-, phone- and state-level alignments.
// To run this example, place a 16-bit, 16kHz, mono wav file with the spoken text "one two three four five six seven eight nine ten" named "audio.wav" in
// the examples/data directory and run it with `cargo run --example alignment`.

use pocketsphinx::{AlignmentIterItem, Config, LogMath};

fn print_alignment_item(item: &AlignmentIterItem, logmath: &LogMath, indent: usize) {
    let name = item.get_name();
    let seg = item.get_seg();
    let start_s = seg.start as f32 / 100.0;
    let duration_s = seg.duration as f32 / 100.0;
    let end_s = start_s + duration_s;
    let score = seg.score;
    let prop = logmath.exp(score);
    let indent_str = " ".repeat(indent);
    println!(
        "{}{} (Prop: {:.2}, Time: {:.2}s -> {:.2}s)",
        indent_str, name, prop, start_s, end_s
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let audio_path = format!("{}/examples/data/audio.wav", manifest_dir);
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

    // For word alignment, we need to set the alignment text before decoding
    decoder.set_align_text("one two three four five six seven eight nine ten")?;

    // Decode audio
    decoder.start_utt()?;
    decoder.process_raw(&audio_i16, false, true)?;
    decoder.end_utt()?;

    // Now we have to do a second decoder pass to get phone- and state-level alignment
    // Set up decoder to run phone and state-level alignment.
    decoder
        .set_alignment(None)
        .expect("Failed to set alignment");

    // Decode audio
    decoder.start_utt()?;
    decoder.process_raw(&audio_i16, false, true)?;
    decoder.end_utt()?;

    // Get the alignment
    let alignment = decoder.get_alignment().expect("Failed to get alignment");

    // We use the decoder logmath to convert the alignment scores to probabilities
    let logmath = decoder.get_logmath();

    // Print the word, phone, and state-level alignments
    for align in alignment.get_words() {
        print_alignment_item(&align, &logmath, 0);
        let phone_align = align.get_children().expect("Failed to get phone alignment");
        for align in phone_align {
            print_alignment_item(&align, &logmath, 2);
            let state_align = align.get_children().expect("Failed to get state alignment");
            for align in state_align {
                print_alignment_item(&align, &logmath, 4);
            }
        }
    }

    // We could also get phone- and state-level alignments directly with `alignment.get_phones()`, `alignment.get_states()`:
    // for align in alignment.get_phones() {
    //     let name = align.get_name();
    //     let seg = align.get_seg();
    //     let start_s = seg.start as f32 / 100.0;
    //     let duration_s = seg.duration as f32 / 100.0;
    //     let end_s = start_s + duration_s;
    //     let score = seg.score;
    //     println!(
    //         "[{:.2} - {:.2}]\t{} (Score: {})",
    //         start_s, end_s, name, score
    //     );
    // }

    Ok(())
}
