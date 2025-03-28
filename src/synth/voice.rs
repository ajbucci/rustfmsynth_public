use super::algorithm::Algorithm;
use super::envelope::EnvelopeGenerator;
use super::note::NoteSource;
use super::operator::Operator;

/// Represents a single polyphonic voice in the synthesizer.
pub struct Voice {
    pub active: bool,                    // Is the voice currently playing a note?
    pub note_number: u8,                 // MIDI note number (0-127)
    pub note_frequency: f32,             // Frequency derived from note_number
    pub note_source: Option<NoteSource>, // Where the note came from (keyboard, sequencer)
    envelope: EnvelopeGenerator,         // Main amplitude envelope for the voice
    samples_elapsed_since_trigger: u64,  // Counter for phase calculation
}

impl Voice {
    /// Creates a new, inactive voice.
    pub fn new() -> Self {
        Self::default()
    }

    /// Activates the voice for a given note.
    /// Resets the sample counter and triggers the envelope.
    pub fn activate(
        &mut self,
        note_number: u8,
        note_source: Option<NoteSource>,
        note_frequency: f32,
    ) {
        self.active = true;
        self.note_number = note_number;
        self.note_source = note_source;
        self.note_frequency = note_frequency;
        self.samples_elapsed_since_trigger = 0;
        self.envelope.trigger();

        println!(
            "Voice activated note {}, sample counter reset",
            self.note_number
        );
        // Trigger the main envelope
        // TODO: pass envelope events to the operator when processing to trigger operator envelopes
        self.envelope.trigger();
    }

    /// Initiates the release phase of the voice's main envelope.
    pub fn release(&mut self) {
        // Check if the voice is actually active OR the envelope is still running before releasing.
        // Avoids re-releasing if multiple note-offs are received or if already released.
        if self.active || !self.envelope.is_finished() {
            println!("Voice releasing envelope for note {}", self.note_number);
            self.envelope.release();

            // Mark the voice as inactive (no longer accepting triggers),
            // but it will continue processing until the envelope finishes its release phase.
            self.active = false;
        }
    }

    /// Processes a buffer of audio for this voice using the provided algorithm and operators.
    /// `algorithm`: The FM algorithm defining operator connections.
    /// `operators`: The set of operators configured in the SynthEngine.
    /// `output`: The buffer to add this voice's contribution to.
    /// `sample_rate`: The audio sample rate.
    pub fn process(
        &mut self,
        algorithm: &Algorithm,  // Pass algorithm
        operators: &[Operator], // Pass operators slice
        output: &mut [f32],     // Note: This should likely be additive or cleared upstream
        sample_rate: f32,
    ) {
        // If the voice is fully finished (inactive AND envelope done), skip processing.
        if self.is_finished() {
            // Ensure output is silent if this voice is the only contributor?
            // Or assume the main engine clears the buffer.
            // output.fill(0.0); // Optional: Clear if needed
            return;
        }

        let buffer_len = output.len();
        if buffer_len == 0 {
            return; // Nothing to process
        }

        // Store the sample index corresponding to the START of this buffer.
        let start_sample_index = self.samples_elapsed_since_trigger;

        // --- Generate Raw Audio using Algorithm and Operators ---
        // Create a temporary buffer for the raw operator output before enveloping.
        let mut raw_output = vec![0.0; buffer_len];
        algorithm.process(
            operators, // Pass the operators slice
            self.note_frequency,
            &mut raw_output, // Generate into the temporary buffer
            sample_rate,
            start_sample_index,
        );

        // --- Apply Main Voice Envelope ---
        // Apply the overall envelope to the raw generated sound.
        self.envelope.apply(&mut raw_output, sample_rate); // Apply modifies raw_output in place

        // --- Add to Final Output ---
        // Add the enveloped sound of this voice to the main output buffer.
        // Assumes the main output buffer might contain other voices.
        for i in 0..buffer_len {
            output[i] += raw_output[i]; // Additive mixing
        }

        // --- Update State & Increment Counter ---

        // Check if the envelope has finished its release phase *after* processing.
        if !self.active && self.envelope.is_finished() {
            // The voice was releasing and the envelope just finished.
            // It's now truly inactive. No state change needed here, is_finished() handles it.
            println!(
                "Envelope finished release for note {}, voice fully inactive",
                self.note_number
            );
        }

        // Increment the sample counter *after* processing this buffer.
        // Only increment if the voice was considered active OR was releasing during this buffer.
        self.samples_elapsed_since_trigger += buffer_len as u64;
    }

    /// Checks if the voice is completely finished (inactive and envelope has finished).
    pub fn is_finished(&self) -> bool {
        // A voice is finished if it's not marked active (i.e., released)
        // AND its envelope has reached the idle state (value is effectively zero).
        !self.active && self.envelope.is_finished()
    }
}
impl Default for Voice {
    fn default() -> Self {
        Self {
            active: false,
            note_number: 0,
            note_frequency: 0.0, // Will be set on activation
            note_source: None,
            envelope: EnvelopeGenerator::new(),
            samples_elapsed_since_trigger: 0,
        }
    }
}
