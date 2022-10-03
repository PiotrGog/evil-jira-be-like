use super::PlayerTrait;
use anyhow;
use rodio::source::{SineWave, Source};
use rodio::{OutputStream, Sink};
use std::time::Duration;

pub struct SinPlayer {
    frequency: f32,
    duration: Duration,
    amplify: f32,
}

impl PlayerTrait for SinPlayer {
    fn play(&self) -> anyhow::Result<()> {
        let (_stream_device, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        let source = SineWave::new(self.frequency)
            .take_duration(self.duration)
            .amplify(self.amplify);
        sink.append(source);

        sink.sleep_until_end();
        Ok(())
    }
}

impl SinPlayer {
    pub fn new() -> Self {
        Self {
            frequency: 440.0,
            duration: Duration::from_secs_f32(5.0),
            amplify: 0.20,
        }
    }
}
