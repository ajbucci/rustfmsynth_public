#[derive(Clone, Debug)]
pub enum FilterType {
    LowPass(f32),       // cutoff frequency
    HighPass(f32),      // cutoff frequency
    BandPass(f32, f32), // center frequency, bandwidth
}

pub fn apply_filter(output: &mut [f32], filter_type: FilterType, sample_rate: f32) {
    match filter_type {
        FilterType::LowPass(cutoff) => apply_low_pass(output, cutoff, sample_rate),
        FilterType::HighPass(cutoff) => apply_high_pass(output, cutoff, sample_rate),
        FilterType::BandPass(center, bandwidth) => {
            apply_band_pass(output, center, bandwidth, sample_rate)
        }
    }
}

// Basic low-pass filter using a simple averaging technique
fn apply_low_pass(output: &mut [f32], cutoff: f32, sample_rate: f32) {
    let rc = 1.0 / (cutoff * 2.0 * std::f32::consts::PI);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (rc + dt);

    let mut previous = output[0];
    for sample in output.iter_mut() {
        *sample = previous + alpha * (*sample - previous);
        previous = *sample;
    }
}

// Basic high-pass filter using a simple high-pass formula
fn apply_high_pass(output: &mut [f32], cutoff: f32, sample_rate: f32) {
    let rc = 1.0 / (cutoff * 2.0 * std::f32::consts::PI);
    let dt = 1.0 / sample_rate;
    let alpha = rc / (rc + dt);

    let mut previous_input = output[0];
    let mut previous_output = output[0];
    for sample in output.iter_mut() {
        let current_input = *sample;
        *sample = alpha * (previous_output + current_input - previous_input);
        previous_input = current_input;
        previous_output = *sample;
    }
}

// Basic band-pass filter by combining low-pass and high-pass
fn apply_band_pass(output: &mut [f32], center: f32, bandwidth: f32, sample_rate: f32) {
    apply_low_pass(output, center + bandwidth / 2.0, sample_rate);
    apply_high_pass(output, center - bandwidth / 2.0, sample_rate);
}
