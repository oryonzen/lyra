use rminiaudio::{Context, Device, DeviceConfig, DeviceType, Format};
use rtrb::{Consumer, Producer, RingBuffer};
use std::{collections::HashMap, io, sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering},}, thread::{self, JoinHandle}, time::{Duration, Instant},};

pub struct LyraProject {
    pub device: Device,
    pub context: Context,
    pub sample_rate: f32,
    pub channel_count: u8,
    pub buffer_size: u32,
    pub is_playing: Arc<AtomicBool>,
    pub master_volume: f32,
    pub bpm: f64,
    pub playhead_position: Arc<AtomicU64>,
    pub loop_start: u64,
    pub loop_end: u64,
    pub is_looping: bool,
    pub mixed_full_buffer: Vec<f32>,
    pub tracks: HashMap<u128, Track>,
    producer: Option<Producer<f32>>,
    worker: Option<JoinHandle<Producer<f32>>>,
    worker_running: Arc<AtomicBool>,
    producer_done: Arc<AtomicBool>,
    ring_empty: Arc<AtomicBool>,
}

pub struct Track {
    pub audio_buffer: Vec<f32>,
    pub volume: f32,
    pub pan: f32,
    pub is_muted: bool,
    pub is_soloed: bool,
}

impl LyraProject {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let context = Context::new()?;
        let (producer, mut consumer) = RingBuffer::<f32>::new(44100 * 2);

        let is_playing = Arc::new(AtomicBool::new(false));
        let playhead_position = Arc::new(AtomicU64::new(0));
        let worker_running = Arc::new(AtomicBool::new(false));
        let producer_done = Arc::new(AtomicBool::new(true));
        let ring_empty = Arc::new(AtomicBool::new(true));

        let callback_playing = Arc::clone(&is_playing);
        let callback_playhead = Arc::clone(&playhead_position);
        let callback_done = Arc::clone(&producer_done);
        let callback_empty = Arc::clone(&ring_empty);

        let device_config = DeviceConfig::new(DeviceType::Playback)
            .sample_rate(44100)
            .channels(2)
            .format(Format::F32)
            .data_callback(move |output, _input| {
                let playing = callback_playing.load(Ordering::Acquire);
                let (filled, remainder) = consumer.pop_partial_slice(output);
                let consumed = filled.len();

                if playing {
                    remainder.fill(0.0);

                    callback_playhead.fetch_add((consumed / 2) as u64, Ordering::Relaxed);

                    if consumer.slots() == 0 && callback_done.load(Ordering::Acquire) {
                        callback_playing.store(false, Ordering::Release);
                    }
                } else {
                    output.fill(0.0);
                }

                callback_empty.store(consumer.slots() == 0, Ordering::Release);
            });

        let device = context.create_device(device_config)?;
        device.start()?;

        Ok(Self {
            device,
            context,
            sample_rate: 44100.0,
            channel_count: 2,
            buffer_size: 512,
            is_playing,
            master_volume: 1.0,
            bpm: 120.0,
            playhead_position,
            loop_start: 0,
            loop_end: 0,
            is_looping: false,
            mixed_full_buffer: vec![0.0; 512 * 2],
            tracks: HashMap::new(),
            producer: Some(producer),
            worker: None,
            worker_running,
            producer_done,
            ring_empty,
        })
    }

    pub fn play(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_playing.load(Ordering::Acquire) {
            return Ok(());
        }

        if let Some(worker) = self.worker.take() {
            self.producer = Some(
                worker
                    .join()
                    .map_err(|_| io::Error::other("Audio producer thread panicked"))?,
            );
        }

        let deadline = Instant::now() + Duration::from_secs(2);

        while !self.ring_empty.load(Ordering::Acquire) {
            if Instant::now() >= deadline {
                return Err(io::Error::other(
                    "Timed out waiting for the audio ring buffer to drain",
                )
                .into());
            }

            thread::yield_now();
        }

        let mut producer = self
            .producer
            .take()
            .ok_or_else(|| io::Error::other("Audio producer is unavailable"))?;

        let source = self.mixed_full_buffer.clone();
        let source_len = source.len() / 2 * 2;
        let mut position = self.playhead_position.load(Ordering::Relaxed) as usize * 2;

        if position >= source_len {
            position = 0;
            self.playhead_position.store(0, Ordering::Relaxed);
        }

        self.worker_running.store(true, Ordering::Release);
        self.producer_done.store(false, Ordering::Release);
        self.is_playing.store(true, Ordering::Release);

        let worker_running = Arc::clone(&self.worker_running);
        let producer_done = Arc::clone(&self.producer_done);

        self.worker = Some(thread::spawn(move || {
            while position < source_len && worker_running.load(Ordering::Acquire) {
                let (written, _) = producer.push_partial_slice(&source[position..source_len]);

                position += written.len();

                if written.is_empty() {
                    thread::yield_now();
                }
            }

            producer_done.store(true, Ordering::Release);
            producer
        }));

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_playing.store(false, Ordering::Release);
        self.worker_running.store(false, Ordering::Release);

        if let Some(worker) = self.worker.take() {
            self.producer = Some(
                worker
                    .join()
                    .map_err(|_| io::Error::other("Audio producer thread panicked"))?,
            );
        }

        self.ring_empty.store(false, Ordering::Release);

        Ok(())
    }
}
