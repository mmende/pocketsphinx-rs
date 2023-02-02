# PocketSphinx

This crate provides a rust wrapper around the stable release of PocketSphinx. [After PocketSphinx finally lost it's prealpha status after a decade or so](https://github.com/cmusphinx/pocketsphinx/releases/tag/v5.0.0), the rust bindings from [kriomant](https://github.com/kriomant/pocketsphinx-rs) were no longer up to date with the slightly modified [PocketSphinx API](https://cmusphinx.github.io/doc/pocketsphinx/) which is why I decided to give it a go and create up to date bindings. Furthermore I wanted PocketSphinx to be linked statically to avoid the need to install the PocketSphinx libraries on the target system.

The crate closely resembles the [C API of PocketSphinx](https://cmusphinx.github.io/doc/pocketsphinx/) in a rusty way. So instead of using `ps_decoder_t` you would use a `Decoder` struct and instead of `ps_start_utt` you would use `Decoder::start_utt`.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
pocketsphinx = { git = "https://github.com/mmende/pocketsphinx-rs.git", version = "0.1.0" }
```

Examples can be found in the `examples` directory.