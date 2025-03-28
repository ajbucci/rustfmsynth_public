mod cpal_backend;

pub use self::cpal_backend::CpalBackend;

pub trait AudioBackend {
    fn new() -> Self;
    fn start(&mut self);
    fn stop(&mut self);
    fn process_audio(&mut self, output: &mut [f32]);
}

