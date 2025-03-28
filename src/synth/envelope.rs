pub struct EnvelopeGenerator {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub value: f32,
    state: EnvelopeState,
    release_start_value: f32,
    min_threshold: f32,
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl EnvelopeGenerator {
    pub fn new() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
            value: 0.0,
            state: EnvelopeState::Idle,
            release_start_value: 0.0,
            min_threshold: 0.001,
        }
    }

    pub fn trigger(&mut self) {
        // println!(
        //     "Envelope trigger: state={:?}, value={}",
        //     self.state, self.value
        // );
        self.state = EnvelopeState::Attack;
        // println!(
        //     "After trigger: state={:?}, value={}",
        //     self.state, self.value
        // );
    }

    pub fn release(&mut self) {
        // println!(
        //     "Envelope release: state={:?}, value={}, release_start={}",
        //     self.state, self.value, self.release_start_value
        // );
        if self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
            self.release_start_value = self.value;
            // println!(
            //     "After release: state={:?}, value={}, release_start={}",
            //     self.state, self.value, self.release_start_value
            // );
        }
    }

    pub fn is_finished(&self) -> bool {
        self.state == EnvelopeState::Idle && self.value == 0.0
    }

    pub fn apply(&mut self, output: &mut [f32], sample_rate: f32) {
        let attack_step = 1.0 / (self.attack * sample_rate);
        let decay_step = (1.0 - self.sustain) / (self.decay * sample_rate);
        let release_step = self.value / (self.release * sample_rate);

        // println!(
        //     "Apply start: state={:?}, value={}, steps: a={}, d={}, r={}",
        //     self.state, self.value, attack_step, decay_step, release_step
        // );

        for sample in output.iter_mut() {
            let old_state = self.state;

            if self.state != EnvelopeState::Idle {
                self.value = match self.state {
                    EnvelopeState::Attack => {
                        self.value += attack_step;
                        if self.value >= 1.0 {
                            self.state = EnvelopeState::Decay;
                            1.0
                        } else {
                            self.value
                        }
                    }
                    EnvelopeState::Decay => {
                        self.value -= decay_step;
                        if self.value <= self.sustain {
                            self.state = EnvelopeState::Sustain;
                            self.sustain
                        } else {
                            self.value
                        }
                    }
                    EnvelopeState::Sustain => self.value,
                    EnvelopeState::Release => {
                        self.value -= release_step;
                        if self.value <= self.min_threshold {
                            self.state = EnvelopeState::Idle;
                            self.value = 0.0;
                            0.0
                        } else {
                            self.value
                        }
                    }
                    EnvelopeState::Idle => 0.0,
                };
            }

            *sample *= self.value;

            // if old_state != self.state {
            //     println!(
            //         // "State transition: {:?} -> {:?}, value={}",
            //         old_state,
            //         self.state, self.value
            //     );
            // }
        }
        // println!("Envelope state={:?}, value={}", self.state, self.value);
    }
}
