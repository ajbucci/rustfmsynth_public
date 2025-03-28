use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
}

#[derive(Debug, Clone)] // Added Debug and Clone
pub struct WaveformGenerator {
    pub waveform: Waveform, // Made public for inspection/logging if needed
}

impl WaveformGenerator {
    pub fn new(waveform: Waveform) -> Self {
        Self { waveform }
    }
    pub fn generate(
        &self,
        frequency: f32,
        sample_rate: f32,
        // TODO: externally should wrap phase_offset if it grows large -- phase_offset = phase_offset % (2.0 * std::f32::consts::PI)
        phase_offset: f32,
        output: &mut [f32],
        modulation: &[f32],
    ) {
        // TODO: this implementation relies on slightly more expensive transcendental functions such as asin()
        // in the future may want to look into modulo arithmetic and other optimizations (PolyBLEP etc.)
        let generate_wave = match self.waveform {
            Waveform::Sine => |phase: f32| phase.sin(),
            Waveform::Square => |phase: f32| if phase.sin() >= 0.0 { 1.0 } else { -1.0 },
            Waveform::Sawtooth => |phase: f32| {
                let cycles = phase / (2.0 * std::f32::consts::PI);
                2.0 * (cycles - (cycles + 0.5).floor())
            },
            Waveform::Triangle => |phase: f32| (2.0 / std::f32::consts::PI) * (phase.sin()).asin(),
            Waveform::Noise => |_phase: f32| rand::thread_rng().gen_range(-1.0..1.0),
        };

        let phase_increment = 2.0 * std::f32::consts::PI * frequency / sample_rate;

        for (i, sample) in output.iter_mut().enumerate() {
            let current_phase = phase_offset + phase_increment * (i as f32);
            *sample = generate_wave(current_phase + modulation[i]);
        }
    }
    pub fn get_next_waveform(&mut self) {
        self.waveform = match self.waveform {
            Waveform::Noise => Waveform::Sine,
            Waveform::Sine => Waveform::Square,
            Waveform::Square => Waveform::Sawtooth,
            Waveform::Sawtooth => Waveform::Triangle,
            Waveform::Triangle => Waveform::Noise,
        };
    }
    pub fn get_previous_waveform(&mut self) {
        self.waveform = match self.waveform {
            Waveform::Noise => Waveform::Triangle,
            Waveform::Sine => Waveform::Noise,
            Waveform::Square => Waveform::Sine,
            Waveform::Sawtooth => Waveform::Square,
            Waveform::Triangle => Waveform::Sawtooth,
        };
    }
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }
}
