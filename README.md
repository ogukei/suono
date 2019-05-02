# Suono
A simple FLAC decoder written in Rust

### How to Try
for converting .flac to .wav

1. Place a `input.flac` file whatever you like on this repository directory. i.e. `~/suono/input.flac`
1. `$ cargo run --release` 
1. `output.wav` will be created on the same directory

NOTE: requires Cargo support Rust 2018 to run the binary.

_Sample .flac files can be found at such as: https://helpguide.sony.net/high-res/sample1/v1/en/index.html_

### Feature
- Decent decoding speed (took 5 seconds @ 3.20GHz, 4 minutes duration track, 110MB .flac)
- Portable (no libraries needed as the basic decoding feature. uses some to output .wav file for exporting the result)

For more information about FLAC, see https://xiph.org/flac/
