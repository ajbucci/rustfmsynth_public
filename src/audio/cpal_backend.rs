use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use std::sync::{Arc, Mutex};
use crate::audio::AudioBackend;
use crate::synth::engine::SynthEngine;

pub struct CpalBackend {
    stream: Option<Stream>,
    synth_engine: Arc<Mutex<SynthEngine>>,
}

impl CpalBackend {
    pub fn new_with_engine(synth_engine: Arc<Mutex<SynthEngine>>) -> Self {
        Self {
            stream: None,
            synth_engine,
        }
    }
    fn determine_buffer_size(&self, device: &cpal::Device, config: cpal::SupportedStreamConfig) -> Result<usize, Box<dyn std::error::Error>> {
        let channels = config.channels() as usize;

        let (buffer_size_sender, buffer_size_receiver) = std::sync::mpsc::channel();

        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let buffer_size = data.len() / channels;
                    buffer_size_sender.send(buffer_size).unwrap();
                },
                |err| eprintln!("an error occurred on stream: {}", err),
                None,
            )?,
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;
        let buffer_size = buffer_size_receiver.recv()?;
        stream.pause()?;

        Ok(buffer_size)
    }

    fn build_stream(&mut self) -> Result<Stream, Box<dyn std::error::Error>> {
        let host = cpal::default_host();

        // Get the appropriate output device based on the target OS
        let device = if cfg!(target_os = "linux") {
            println!("Linux detected, searching for devices");
            if let Ok(devices) = host.devices() {
                let mut device_names = Vec::new();

                for device in devices {
                    let name = device.name().unwrap_or_default();
                    println!("Device: {}", name);
                    if name.to_lowercase().starts_with("default:") || name.to_lowercase().contains("pipewire") {
                        device_names.push(name.clone());
                    }
                }

                println!("Select output device:");
                for (i, name) in device_names.iter().enumerate() {
                    println!("{}. {}", i + 1, name);
                }

                let mut choice = String::new();
                std::io::stdin().read_line(&mut choice).expect("Failed to read input");
                let choice = choice.trim().parse::<usize>().unwrap_or(0);

                // Find the device by name
                let selected_name = if choice > 0 && choice <= device_names.len() {
                    Some(device_names[choice - 1].clone())
                } else {
                    None
                };

                if let Some(name) = selected_name {
                    host.devices()?
                        .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                        .ok_or("No output device available")?
                } else {
                    host.default_output_device().ok_or("No output device available")?
                }
            } else {
                host.default_output_device().ok_or("No output device available")?
            }
        } else {
            // For non-Linux platforms, use the default output device
            host.default_output_device().ok_or("No output device available")?
        };
        println!("Selected device: {}", device.name().unwrap_or_default());
        let config = device.default_output_config()?;
        let buffer_config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let buffer_size = self.determine_buffer_size(&device, buffer_config)?;

        {
            let mut synth_engine = self.synth_engine.lock().unwrap();
            synth_engine.set_buffer_size(buffer_size);
        }

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let synth_engine = self.synth_engine.clone();

        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut synth_engine = synth_engine.lock().unwrap();
                    let mut buffer = vec![0.0; data.len() / channels];
                    synth_engine.process(&mut buffer, sample_rate as f32);

                    for (i, frame) in data.chunks_mut(channels).enumerate() {
                        for sample in frame.iter_mut() {
                            *sample = buffer[i];
                        }
                    }
                },
                |err| eprintln!("an error occurred on stream: {}", err),
                None,
            )?,
            _ => return Err("Unsupported sample format".into()),
        };

        Ok(stream)
    }
}

impl AudioBackend for CpalBackend {
    fn new() -> Self {
        Self {
            stream: None,
            synth_engine: Arc::new(Mutex::new(SynthEngine::new())),
        }
    }

    fn start(&mut self) {
        if let Ok(stream) = self.build_stream() {
            stream.play().expect("Failed to play stream");
            self.stream = Some(stream);
        }
    }

    fn stop(&mut self) {
        if let Some(stream) = &self.stream {
            stream.pause().expect("Failed to pause stream");
        }
    }

    fn process_audio(&mut self, output: &mut [f32]) {
        let mut synth_engine = self.synth_engine.lock().unwrap();
        synth_engine.process(output, 44100.0);
    }
}
