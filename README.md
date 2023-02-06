# PocketSphinx

This crate provides a rust wrapper around the stable release of PocketSphinx. [After PocketSphinx finally lost it's prealpha status after a decade or so](https://github.com/cmusphinx/pocketsphinx/releases/tag/v5.0.0), the rust bindings from [kriomant](https://github.com/kriomant/pocketsphinx-rs) were no longer up to date with the slightly modified [PocketSphinx API](https://cmusphinx.github.io/doc/pocketsphinx/) which is why I decided to give it a go and create up to date bindings. Furthermore I wanted PocketSphinx to be linked statically to avoid the need to install the PocketSphinx libraries on the target system.

The crate closely resembles the [C API of PocketSphinx](https://cmusphinx.github.io/doc/pocketsphinx/) in a rusty way. So instead of using `ps_decoder_t` you would use a `Decoder` struct and instead of `ps_start_utt` you would use `Decoder::start_utt` and so on.

## Usage

The crate is currently not published on crates.io, so you have to use it as a git dependency.

Add this to your `Cargo.toml`:

```toml
[dependencies]
pocketsphinx = { git = "https://github.com/mmende/pocketsphinx-rs.git", version = "0.1.0" }
```

Then simply create a config and a decoder and start decoding:

```rust
let mut config = Config::default()?;

let mut decoder = config.init_decoder()?;
decoder.start_utt()?;
decoder.process_raw(&audio_i16, false, false)?;
decoder.end_utt()?;

match decoder.get_hyp()? {
    Some((hyp, _score)) => println!("Hypothesis: {}", hyp),
    None => println!("No hypothesis"),
}
```

## Examples

Examples can be found in the `examples` directory.

- [x] [Default-LM-Recognition](examples/file_default.rs)
- [x] [JSGF-Recognition](examples/file_jsgf.rs)
- [x] [Segment-Iterator](examples/segments.rs)
- [x] [Microphone-Recognition (with endpointing)](examples/live.rs)
- [ ]  ¯\\_(ツ)_/¯ - You tell me

## Roadmap

**Implementation**

- [x] Config
- [x] Decoder
- [x] NBest-Iterator
- [x] Seg-Iterator
- [x] Search-Iterator
- [x] Endpointing / VAD
- [x] Alignment
- [x] Logmath
- [x] FSG
- [ ] JSGF (partially implemented in decoder)
- [ ] KWS (partially implemented in decoder)
- [ ] N-Gram
- [ ] Latice
- [ ] MLLR