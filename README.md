# An FM Synthesizer written in Rust

Currently supports using a QWERTY keyboard to trigger notes. Keys A through ; on the home row correspond to natural notes. Sharps and flats can be found on the QWERTY row.

Users can cycle waveforms using the "," and "." keys on their keyboard.

## Running

Tested on MacOS and Linux.

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repo
git clone https://github.com/ajbucci/rustfmsynth_public.git

# Change directory
cd rustfmsynth_public

# Build/run debug version
cargo run

# Optionally build and run --release for more performance
cargo build --release
./target/release/rustfmsynth
```
