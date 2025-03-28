use crate::synth::engine::SynthEngine;
use crate::synth::note::{NoteEvent, NoteSource};
use crate::synth::operator::{CycleDirection, OperatorEvent};
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::collections::HashMap;

pub struct KeyboardHandler {
    device_state: DeviceState,
    key_states: HashMap<Keycode, bool>,
    key_to_note: HashMap<Keycode, u8>,
    control_keys: HashMap<Keycode, bool>, // Track control keys separately
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, engine: &mut SynthEngine) {
        let keys: Vec<Keycode> = self.device_state.get_keys();
        let note_sender = engine.get_note_sender();
        let operator_sender = engine.get_operator_sender();

        // Check each mapped key for notes
        for (key, note) in &self.key_to_note {
            let is_pressed = keys.contains(key);
            let was_pressed = self.key_states.get(key).cloned().unwrap_or(false);

            if is_pressed != was_pressed {
                if is_pressed {
                    println!(
                        "Key '{:?}' pressed - sending note on for note {}",
                        key, note
                    );
                    if let Ok(event) = NoteEvent::new(*note, 100, true, NoteSource::Keyboard) {
                        if let Err(e) = note_sender.send(event) {
                            eprintln!("Error sending note on event: {}", e);
                        }
                    }
                } else {
                    println!(
                        "Key '{:?}' released - sending note off for note {}",
                        key, note
                    );
                    if let Ok(event) = NoteEvent::new(*note, 0, false, NoteSource::Keyboard) {
                        if let Err(e) = note_sender.send(event) {
                            eprintln!("Error sending note off event: {}", e);
                        }
                    }
                }
                self.key_states.insert(*key, is_pressed);
            }
        }

        // Check control keys for waveform cycling
        for key in [Keycode::Comma, Keycode::Dot].iter() {
            let is_pressed = keys.contains(key);
            let was_pressed = self.control_keys.get(key).cloned().unwrap_or(false);

            if is_pressed && !was_pressed {
                // Key just pressed
                match key {
                    Keycode::Comma => {
                        println!("Cycling waveform backward");
                        if let Err(e) = operator_sender.send(OperatorEvent::CycleWaveform {
                            direction: CycleDirection::Backward,
                        }) {
                            eprintln!("Error sending operator event: {}", e);
                        }
                    }
                    Keycode::Dot => {
                        println!("Cycling waveform forward");
                        if let Err(e) = operator_sender.send(OperatorEvent::CycleWaveform {
                            direction: CycleDirection::Forward,
                        }) {
                            eprintln!("Error sending operator event: {}", e);
                        }
                    }
                    _ => {}
                }
            }

            self.control_keys.insert(*key, is_pressed);
        }
    }
}
impl Default for KeyboardHandler {
    fn default() -> Self {
        let device_state = DeviceState::new();
        let mut key_states: HashMap<Keycode, bool> = HashMap::new();
        let mut control_keys: HashMap<Keycode, bool> = HashMap::new();

        // Define keyboard to note mapping
        let key_to_note: HashMap<Keycode, u8> = [
            // Bottom row - natural notes (A, B, C, D, E, F, G, A, B, C)
            (Keycode::A, 69),         // A4
            (Keycode::S, 71),         // B4
            (Keycode::D, 72),         // C5
            (Keycode::F, 74),         // D5
            (Keycode::G, 76),         // E5
            (Keycode::H, 77),         // F5
            (Keycode::J, 79),         // G5
            (Keycode::K, 81),         // A5
            (Keycode::L, 83),         // B5
            (Keycode::Semicolon, 84), // C6
            // Top row - sharp/flat notes (A#, C#, D#, F#, G#, A#)
            (Keycode::W, 70),           // A#4/Bb4
            (Keycode::R, 73),           // C#5/Db5
            (Keycode::T, 75),           // D#5/Eb5
            (Keycode::U, 78),           // F#5/Gb5
            (Keycode::I, 80),           // G#5/Ab5
            (Keycode::O, 82),           // A#5/Bb5
            (Keycode::LeftBracket, 85), // C#6/Db6
        ]
        .iter()
        .cloned()
        .collect();

        // Initialize all keys as not pressed
        for key in key_to_note.keys() {
            key_states.insert(*key, false);
        }

        // Initialize control keys
        control_keys.insert(Keycode::Comma, false);
        control_keys.insert(Keycode::Dot, false);

        Self {
            device_state,
            key_states,
            key_to_note,
            control_keys,
        }
    }
}
