# PocketSphinx

This crate provides a rust wrapper around the stable release of PocketSphinx. [After PocketSphinx finally lost it's prealpha status after a decade or so](https://github.com/cmusphinx/pocketsphinx/releases/tag/v5.0.0), the rust bindings from [kriomant](https://github.com/kriomant/pocketsphinx-rs) were no longer up to date with the slightly modified [PocketSphinx API](https://cmusphinx.github.io/doc/pocketsphinx/) which is why I decided to give it a go and create up to date bindings. Furthermore I wanted PocketSphinx to be linked statically to avoid the need to install the PocketSphinx libraries on the target system.

The crate closely resembles the [C API of PocketSphinx](https://cmusphinx.github.io/doc/pocketsphinx/) in a rusty way. So instead of using `ps_decoder_t` you would use a `Decoder` struct and instead of `ps_start_utt` you would use `Decoder::start_utt`.

## Usage

The crate is currently not published on crates.io, so you have to use it as a git dependency.

Add this to your `Cargo.toml`:

```toml
[dependencies]
pocketsphinx = { git = "https://github.com/mmende/pocketsphinx-rs.git", version = "0.1.0" }
```

Then simply create a config and a decoder and start decoding:

```rust
let mut config = Config::new()?;
config.default_search_args();

let mut decoder = config.init_decoder()?;
decoder.start_utt()?;
decoder.process_raw(&audio_i16, false, false)?;
decoder.end_utt()?;

let (hyp, _score) = decoder.get_hyp()?;
println!("Hypothesis: {}", hyp);
```

Examples can be found in the `examples` directory.

## Roadmap

**Implementation**

- [x] Config
- [x] Decoder
- [x] NBest-Iterator
- [x] Seg-Iterator
- [x] Search-Iterator
- [x] Endpointing / VAD
- [ ] FSG
- [ ] JSGF (partially implemented in decoder)
- [ ] KWS
- [ ] N-Gram
- [ ] Alignment
- [ ] Latice
- [ ] MLLR
- [ ] Logmath

**Examples**

- [x] Default-LM-Recognition
- [x] JSGF-Recognition
- [ ] Microphone-Recognition (with endpointing)
- [ ]  ¯\\_(ツ)_/¯ - You tell me