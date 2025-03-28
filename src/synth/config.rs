#[derive(Clone)]
pub struct SynthConfig {
    pub max_voices: usize,
    pub operators_per_voice: usize,
    pub sample_rate: f32,
}

impl Default for SynthConfig {
    fn default() -> Self {
        Self {
            max_voices: 128,
            operators_per_voice: 12,
            sample_rate: 44100.0, // Standard audio sample rate
        }
    }
}
