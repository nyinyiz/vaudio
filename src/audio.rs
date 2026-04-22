use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::SyncSender;

pub struct AudioCapture {
    _stream: cpal::Stream,
}

impl AudioCapture {
    pub fn new(device_name: Option<String>, tx: SyncSender<Vec<f32>>) -> Result<Self> {
        let host = cpal::default_host();

        let device = if let Some(name) = device_name {
            host.input_devices()?
                .find(|d| d.name().map(|n| n.contains(&name)).unwrap_or(false))
                .ok_or_else(|| anyhow!("Could not find audio device containing '{}'", name))?
        } else {
            host.default_input_device()
                .ok_or_else(|| anyhow!("No default input device found"))?
        };

        let config = device.default_input_config()?;
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into(), tx)?,
            cpal::SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into(), tx)?,
            cpal::SampleFormat::U16 => Self::build_stream::<u16>(&device, &config.into(), tx)?,
            _ => return Err(anyhow!("Unsupported sample format")),
        };

        stream.play()?;

        Ok(Self { _stream: stream })
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        tx: SyncSender<Vec<f32>>,
    ) -> Result<cpal::Stream>
    where
        T: cpal::Sample + Into<f32> + cpal::SizedSample,
    {
        let err_callback = |err| eprintln!("An error occurred on the audio stream: {}", err);

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &_| {
                let samples: Vec<f32> = data.iter().map(|&s| s.into()).collect();
                let _ = tx.try_send(samples);
            },
            err_callback,
            None,
        )?;

        Ok(stream)
    }
}

pub fn list_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let devices = host.input_devices()?;
    let mut names = Vec::new();
    for (i, device) in devices.enumerate() {
        if let Ok(name) = device.name() {
            names.push(format!("{}: {}", i, name));
        }
    }
    Ok(names)
}
