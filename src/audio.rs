use crate::spectrum::FFT_SIZE;
use rodio::{OutputStream, Sink, Source};
use std::{
    io::{self, Read, Seek, SeekFrom},
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use symphonia::core::{
    audio::SampleBuffer,
    codecs::DecoderOptions,
    errors::Error as DecodeError,
    formats::{FormatOptions, FormatReader},
    io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
    probe::Hint,
};

pub const STREAM_URL: &str = "https://stream.doomsday.radio/live";

#[derive(Clone, Debug)]
pub enum Status {
    Connecting,
    Live,
    Buffering,
    Retry(String),
}

pub struct Snapshot {
    pub samples: [f32; FFT_SIZE],
    pub sample_rate: u32,
    pub updated: Instant,
}

pub struct Session {
    pub cancelled: AtomicBool,
    pub status: Mutex<Status>,
    pub snapshot: Mutex<Snapshot>,
    volume: AtomicU32,
}

impl Session {
    fn new(volume: f32) -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            status: Mutex::new(Status::Connecting),
            snapshot: Mutex::new(Snapshot {
                samples: [0.0; FFT_SIZE],
                sample_rate: 44_100,
                updated: Instant::now(),
            }),
            volume: AtomicU32::new(volume.to_bits()),
        }
    }

    fn set_status(&self, status: Status) {
        *self.status.lock().unwrap() = status;
    }

    pub fn set_volume(&self, volume: f32) {
        self.volume
            .store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }
}

#[derive(Default)]
pub struct Player {
    pub session: Option<Arc<Session>>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Player {
    pub fn start(&mut self, volume: f32) {
        self.stop();
        let session = Arc::new(Session::new(volume));
        self.session = Some(session.clone());
        self.worker = Some(thread::spawn(move || {
            while !session.cancelled.load(Ordering::Relaxed) {
                session.set_status(Status::Connecting);
                let result = play_stream(session.clone());
                if session.cancelled.load(Ordering::Relaxed) {
                    break;
                }
                session.set_status(Status::Retry(
                    result.err().unwrap_or_else(|| "Stream beendet".into()),
                ));
                thread::park_timeout(Duration::from_secs(3));
            }
        }));
    }

    pub fn stop(&mut self) {
        if let Some(session) = self.session.take() {
            session.cancelled.store(true, Ordering::Relaxed);
        }
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

struct HttpSource {
    runtime: tokio::runtime::Runtime,
    response: Mutex<reqwest::Response>,
    chunk: Vec<u8>,
    position: usize,
    session: Arc<Session>,
}

async fn cancelled(session: &Session) {
    while !session.cancelled.load(Ordering::Relaxed) {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

impl HttpSource {
    fn open(session: Arc<Session>) -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let response = runtime.block_on(async {
            let client = reqwest::Client::builder()
                .https_only(true)
                .connect_timeout(Duration::from_secs(8))
                .redirect(reqwest::redirect::Policy::limited(3))
                .user_agent(concat!("DoomsdayRadio/", env!("CARGO_PKG_VERSION")))
                .build().map_err(|error| error.to_string())?;
            tokio::select! {
                _ = cancelled(&session) => Err("Gestoppt".into()),
                result = tokio::time::timeout(Duration::from_secs(10), client.get(STREAM_URL)
                    .header("Accept-Encoding", "identity").header("Icy-MetaData", "0").send()) => {
                    result.map_err(|_| "Verbindung dauert zu lange".to_string())?
                        .and_then(reqwest::Response::error_for_status).map_err(|error| error.to_string())
                }
            }
        })?;
        Ok(Self {
            runtime,
            response: Mutex::new(response),
            chunk: Vec::new(),
            position: 0,
            session,
        })
    }
}

impl Read for HttpSource {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        if self.session.cancelled.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Gestoppt"));
        }
        while self.position == self.chunk.len() {
            let response = self.response.get_mut().unwrap();
            let next = self.runtime.block_on(async {
                tokio::select! {
                    _ = cancelled(&self.session) => Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Gestoppt")),
                    result = tokio::time::timeout(Duration::from_secs(8), response.chunk()) => {
                        result.map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Keine Streamdaten"))?
                            .map_err(io::Error::other)
                    }
                }
            })?;
            match next {
                Some(chunk) => {
                    self.chunk = chunk.to_vec();
                    self.position = 0;
                }
                None => return Ok(0),
            }
        }
        let count = buffer.len().min(self.chunk.len() - self.position);
        buffer[..count].copy_from_slice(&self.chunk[self.position..self.position + count]);
        self.position += count;
        Ok(count)
    }
}

impl Seek for HttpSource {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Livestream ist nicht seekbar",
        ))
    }
}

impl MediaSource for HttpSource {
    fn is_seekable(&self) -> bool {
        false
    }
    fn byte_len(&self) -> Option<u64> {
        None
    }
}

type DecodedStream = (
    Box<dyn FormatReader>,
    Box<dyn symphonia::core::codecs::Decoder>,
    u32,
);

fn open_decoder(session: Arc<Session>) -> Result<DecodedStream, String> {
    let source = HttpSource::open(session)?;
    let stream = MediaSourceStream::new(Box::new(source), MediaSourceStreamOptions::default());
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|error| error.to_string())?;
    let track = probed.format.default_track().ok_or("Keine Audiospur")?;
    let track_id = track.id;
    let decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|error| error.to_string())?;
    Ok((probed.format, decoder, track_id))
}

fn send_pcm(sender: &SyncSender<Vec<f32>>, mut samples: Vec<f32>, session: &Session) -> bool {
    loop {
        if session.cancelled.load(Ordering::Relaxed) {
            return false;
        }
        match sender.try_send(samples) {
            Ok(()) => return true,
            Err(TrySendError::Full(returned)) => {
                samples = returned;
                thread::park_timeout(Duration::from_millis(4));
            }
            Err(TrySendError::Disconnected(_)) => return false,
        }
    }
}

fn play_stream(session: Arc<Session>) -> Result<(), String> {
    let (_output, handle) =
        OutputStream::try_default().map_err(|error| format!("Audioausgabe: {error}"))?;
    let sink = Sink::try_new(&handle).map_err(|error| error.to_string())?;
    let (mut format, mut decoder, track_id) = open_decoder(session.clone())?;
    let (sender, receiver) = mpsc::sync_channel(12);
    let mut receiver = Some(receiver);
    let mut specification = None;
    while !session.cancelled.load(Ordering::Relaxed) {
        let packet = format.next_packet().map_err(|error| error.to_string())?;
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(DecodeError::DecodeError(_)) => continue,
            Err(error) => return Err(error.to_string()),
        };
        let spec = *decoded.spec();
        if let Some(previous) = specification {
            if spec != previous {
                return Err("Audioformat geaendert".into());
            }
        } else {
            specification = Some(spec);
        }
        let mut samples = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        samples.copy_interleaved_ref(decoded);
        if samples.samples().is_empty() {
            continue;
        }
        if !send_pcm(&sender, samples.samples().to_vec(), &session) {
            break;
        }
        if let Some(receiver) = receiver.take() {
            sink.append(LiveSource::new(
                receiver,
                session.clone(),
                spec.channels.count() as u16,
                spec.rate,
            ));
        }
    }
    Ok(())
}

struct LiveSource {
    receiver: Receiver<Vec<f32>>,
    session: Arc<Session>,
    current: Vec<f32>,
    position: usize,
    channels: u16,
    sample_rate: u32,
    history: [f32; FFT_SIZE],
    history_position: usize,
    channel_position: u16,
    mono: f32,
    live: bool,
}

impl LiveSource {
    fn new(
        receiver: Receiver<Vec<f32>>,
        session: Arc<Session>,
        channels: u16,
        sample_rate: u32,
    ) -> Self {
        Self {
            receiver,
            session,
            current: Vec::new(),
            position: 0,
            channels,
            sample_rate,
            history: [0.0; FFT_SIZE],
            history_position: 0,
            channel_position: 0,
            mono: 0.0,
            live: false,
        }
    }
}

impl Iterator for LiveSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.session.cancelled.load(Ordering::Relaxed) {
            return None;
        }
        let sample = if self.position < self.current.len() {
            let sample = self.current[self.position];
            self.position += 1;
            sample
        } else if self.channel_position != 0 {
            0.0
        } else {
            match self.receiver.try_recv() {
                Ok(samples) if !samples.is_empty() => {
                    self.current = samples;
                    self.position = 1;
                    if !self.live {
                        self.session.set_status(Status::Live);
                        self.live = true;
                    }
                    self.current[0]
                }
                Err(TryRecvError::Disconnected) => return None,
                _ => {
                    if self.live {
                        self.session.set_status(Status::Buffering);
                        self.live = false;
                    }
                    0.0
                }
            }
        };
        let gain = f32::from_bits(self.session.volume.load(Ordering::Relaxed));
        let audible = sample * gain;
        self.mono += audible;
        self.channel_position += 1;
        if self.channel_position == self.channels {
            self.history[self.history_position] = self.mono / self.channels as f32;
            self.history_position += 1;
            self.channel_position = 0;
            self.mono = 0.0;
            if self.history_position == FFT_SIZE {
                if let Ok(mut snapshot) = self.session.snapshot.try_lock() {
                    snapshot.samples.copy_from_slice(&self.history);
                    snapshot.sample_rate = self.sample_rate;
                    snapshot.updated = Instant::now();
                }
                self.history_position = 0;
            }
        }
        Some(audible)
    }
}

impl Source for LiveSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.channels
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_underflow_does_not_block_and_stop_is_immediate() {
        let (_sender, receiver) = mpsc::sync_channel(2);
        let session = Arc::new(Session::new(1.0));
        let mut source = LiveSource::new(receiver, session.clone(), 2, 44_100);
        assert_eq!(source.next(), Some(0.0));
        session.cancelled.store(true, Ordering::Relaxed);
        assert_eq!(source.next(), None);
    }

    #[test]
    fn source_applies_volume_and_taps_consumed_stereo_samples() {
        let (sender, receiver) = mpsc::sync_channel(2);
        sender.send(vec![0.8; FFT_SIZE * 2]).unwrap();
        let session = Arc::new(Session::new(0.5));
        let mut source = LiveSource::new(receiver, session.clone(), 2, 44_100);
        for _ in 0..FFT_SIZE * 2 {
            assert_eq!(source.next(), Some(0.4));
        }
        assert!(session
            .snapshot
            .lock()
            .unwrap()
            .samples
            .iter()
            .all(|sample| *sample == 0.4));
        assert!(matches!(*session.status.lock().unwrap(), Status::Live));
        assert_eq!(source.next(), Some(0.0));
        assert!(matches!(*session.status.lock().unwrap(), Status::Buffering));
        session.set_volume(0.0);
        sender.send(vec![0.8; 4]).unwrap();
        assert_eq!(source.next(), Some(0.0));
    }

    #[test]
    fn closed_channel_ends_playback() {
        let (sender, receiver) = mpsc::sync_channel(2);
        drop(sender);
        let mut source = LiveSource::new(receiver, Arc::new(Session::new(1.0)), 2, 44_100);
        assert_eq!(source.next(), None);
    }

    #[test]
    fn underflow_preserves_stereo_channel_alignment() {
        let (sender, receiver) = mpsc::sync_channel(2);
        let mut source = LiveSource::new(receiver, Arc::new(Session::new(1.0)), 2, 44_100);
        assert_eq!(source.next(), Some(0.0));
        sender.send(vec![0.25, 0.75]).unwrap();
        assert_eq!(source.next(), Some(0.0));
        assert_eq!(source.next(), Some(0.25));
        assert_eq!(source.next(), Some(0.75));
    }

    #[test]
    fn cancelled_producer_does_not_wait_on_full_buffer() {
        let (sender, _receiver) = mpsc::sync_channel(1);
        sender.send(vec![0.0]).unwrap();
        let session = Session::new(1.0);
        session.cancelled.store(true, Ordering::Relaxed);
        assert!(!send_pcm(&sender, vec![1.0], &session));
    }

    #[test]
    #[ignore = "Requires a live internet connection to Doomsday Radio"]
    fn live_stream_decodes_real_mp3_without_seeking() {
        let session = Arc::new(Session::new(1.0));
        let (mut format, mut decoder, track_id) = open_decoder(session).unwrap();
        let mut frames = 0;
        let mut peak = 0.0_f32;
        for _ in 0..100 {
            let packet = format.next_packet().unwrap();
            if packet.track_id() != track_id {
                continue;
            }
            if let Ok(decoded) = decoder.decode(&packet) {
                assert_eq!(decoded.spec().rate, 44_100);
                assert_eq!(decoded.spec().channels.count(), 2);
                let mut samples =
                    SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                samples.copy_interleaved_ref(decoded);
                peak = samples
                    .samples()
                    .iter()
                    .map(|sample| sample.abs())
                    .fold(peak, f32::max);
                frames += samples.len();
            }
        }
        assert!(frames > 44_100);
        assert!(peak > 0.001, "Live stream produced only silence");
    }

    #[test]
    #[ignore = "Requires internet and a working audio output device"]
    fn live_audio_output_consumes_samples_and_worker_stops() {
        let mut player = Player::default();
        player.start(0.1);
        let session = player.session.as_ref().unwrap().clone();
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let heard = session
                .snapshot
                .lock()
                .unwrap()
                .samples
                .iter()
                .any(|sample| sample.abs() > 0.0001);
            if heard {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "No output samples; status: {:?}",
                session.status.lock().unwrap()
            );
            thread::park_timeout(Duration::from_millis(10));
        }
        let worker = player.worker.take().unwrap();
        player.stop();
        worker.thread().unpark();
        let deadline = Instant::now() + Duration::from_secs(3);
        while !worker.is_finished() {
            assert!(
                Instant::now() < deadline,
                "Worker did not stop after cancellation"
            );
            thread::park_timeout(Duration::from_millis(10));
        }
        worker.join().unwrap();
    }
}
