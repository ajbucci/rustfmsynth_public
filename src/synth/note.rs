use std::fmt;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy)]
pub struct NoteEvent {
    pub note_number: u8,
    pub velocity: u8,
    pub is_on: bool,
    pub frequency: f32,
    pub source: NoteSource,
}

impl NoteEvent {
    pub fn new(note_number: u8, velocity: u8, is_on: bool, source: NoteSource) -> Result<Self, NoteError> {
        // Validate note number
        if note_number >= 128 {
            return Err(NoteError::InvalidNoteNumber(note_number));
        }
        
        // Validate velocity
        if velocity >= 128 {
            return Err(NoteError::InvalidVelocity(velocity));
        }
        
        // Get frequency from lookup table
        let frequency = midi_frequencies()[note_number as usize];
        
        Ok(Self { note_number, velocity, is_on, frequency, source })
    }
    
    pub fn validate(&self) -> Result<(), NoteError> {
        if self.note_number >= 128 {
            return Err(NoteError::InvalidNoteNumber(self.note_number));
        }
        
        if self.velocity >= 128 {
            return Err(NoteError::InvalidVelocity(self.velocity));
        }
        
        Ok(())
    }
}

/// Global frequency table for MIDI notes
fn midi_frequencies() -> &'static [f32; 128] {
    static FREQUENCIES: OnceLock<[f32; 128]> = OnceLock::new();
    
    FREQUENCIES.get_or_init(|| {
        let mut frequencies = [0.0; 128];
        for note in 0..128 {
            frequencies[note] = 440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0);
        }
        frequencies
    })
}

#[derive(Debug)]
pub enum NoteError {
    InvalidNoteNumber(u8),
    InvalidVelocity(u8),
}

impl fmt::Display for NoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteError::InvalidNoteNumber(n) => write!(f, "Invalid MIDI note number: {}. Must be 0-127.", n),
            NoteError::InvalidVelocity(v) => write!(f, "Invalid MIDI velocity: {}. Must be 0-127.", v),
        }
    }
}

impl std::error::Error for NoteError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteSource {
    Sequencer,
    Keyboard,
    // Add other sources as needed
}
