// HTML5 Media Element Support (Video/Audio)
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MediaElement {
    pub element_type: MediaType,
    pub src: String,
    pub current_time: f64,
    pub duration: f64,
    pub paused: bool,
    pub volume: f64,
    pub playback_rate: f64,
    pub ready_state: ReadyState,
    pub network_state: NetworkState,
    pub buffered: Vec<TimeRange>,
    pub seekable: Vec<TimeRange>,
    pub ended: bool,
    pub auto_play: bool,
    pub loop_playback: bool,
    pub muted: bool,
    pub controls: bool,
    pub preload: PreloadType,
    pub poster: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub tracks: Vec<TextTrack>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Video,
    Audio,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadyState {
    HaveNothing = 0,
    HaveMetadata = 1,
    HaveCurrentData = 2,
    HaveFutureData = 3,
    HaveEnoughData = 4,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkState {
    Empty = 0,
    Idle = 1,
    Loading = 2,
    NoSource = 3,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreloadType {
    None,
    Metadata,
    Auto,
}

#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone)]
pub struct TextTrack {
    pub kind: TextTrackKind,
    pub label: String,
    pub language: String,
    pub mode: TextTrackMode,
    pub cues: Vec<TextTrackCue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextTrackKind {
    Subtitles,
    Captions,
    Descriptions,
    Chapters,
    Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextTrackMode {
    Disabled,
    Hidden,
    Showing,
}

#[derive(Debug, Clone)]
pub struct TextTrackCue {
    pub id: String,
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct MediaCodec {
    pub name: String,
    pub mime_type: String,
    pub supported: bool,
}

pub struct MediaEngine {
    codecs: HashMap<String, MediaCodec>,
    media_elements: Vec<MediaElement>,
}

impl MediaEngine {
    pub fn new() -> Self {
        let mut engine = MediaEngine {
            codecs: HashMap::new(),
            media_elements: Vec::new(),
        };
        engine.register_default_codecs();
        engine
    }

    fn register_default_codecs(&mut self) {
        self.register_codec("H.264", "video/mp4; codecs=\"avc1.42E01E\"");
        self.register_codec("H.265", "video/mp4; codecs=\"hev1.1.6.L93.B0\"");
        self.register_codec("VP8", "video/webm; codecs=\"vp8\"");
        self.register_codec("VP9", "video/webm; codecs=\"vp9\"");
        self.register_codec("AV1", "video/mp4; codecs=\"av01.0.05M.08\"");
        self.register_codec("AAC", "audio/mp4; codecs=\"mp4a.40.2\"");
        self.register_codec("MP3", "audio/mpeg");
        self.register_codec("Opus", "audio/webm; codecs=\"opus\"");
        self.register_codec("Vorbis", "audio/webm; codecs=\"vorbis\"");
        self.register_codec("FLAC", "audio/flac");
        self.register_codec("WAV", "audio/wav");
    }

    fn register_codec(&mut self, name: &str, mime_type: &str) {
        self.codecs.insert(
            name.to_string(),
            MediaCodec {
                name: name.to_string(),
                mime_type: mime_type.to_string(),
                supported: true,
            },
        );
    }

    pub fn can_play_type(&self, mime_type: &str) -> String {
        for codec in self.codecs.values() {
            if codec.mime_type.contains(mime_type) {
                return "probably".to_string();
            }
        }
        "".to_string()
    }

    pub fn create_media_element(&mut self, media_type: MediaType) -> usize {
        let element = MediaElement {
            element_type: media_type,
            src: String::new(),
            current_time: 0.0,
            duration: 0.0,
            paused: true,
            volume: 1.0,
            playback_rate: 1.0,
            ready_state: ReadyState::HaveNothing,
            network_state: NetworkState::Empty,
            buffered: Vec::new(),
            seekable: Vec::new(),
            ended: false,
            auto_play: false,
            loop_playback: false,
            muted: false,
            controls: false,
            preload: PreloadType::Auto,
            poster: None,
            width: None,
            height: None,
            tracks: Vec::new(),
        };
        
        self.media_elements.push(element);
        self.media_elements.len() - 1
    }

    pub fn load(&mut self, element_id: usize, src: &str) -> Result<(), String> {
        if let Some(element) = self.media_elements.get_mut(element_id) {
            element.src = src.to_string();
            element.network_state = NetworkState::Loading;
            element.ready_state = ReadyState::HaveMetadata;
            Ok(())
        } else {
            Err("Invalid media element".to_string())
        }
    }

    pub fn play(&mut self, element_id: usize) -> Result<(), String> {
        if let Some(element) = self.media_elements.get_mut(element_id) {
            element.paused = false;
            Ok(())
        } else {
            Err("Invalid media element".to_string())
        }
    }

    pub fn pause(&mut self, element_id: usize) -> Result<(), String> {
        if let Some(element) = self.media_elements.get_mut(element_id) {
            element.paused = true;
            Ok(())
        } else {
            Err("Invalid media element".to_string())
        }
    }

    pub fn seek(&mut self, element_id: usize, time: f64) -> Result<(), String> {
        if let Some(element) = self.media_elements.get_mut(element_id) {
            element.current_time = time.max(0.0).min(element.duration);
            Ok(())
        } else {
            Err("Invalid media element".to_string())
        }
    }

    pub fn set_volume(&mut self, element_id: usize, volume: f64) -> Result<(), String> {
        if let Some(element) = self.media_elements.get_mut(element_id) {
            element.volume = volume.max(0.0).min(1.0);
            Ok(())
        } else {
            Err("Invalid media element".to_string())
        }
    }

    pub fn add_text_track(&mut self, element_id: usize, track: TextTrack) -> Result<(), String> {
        if let Some(element) = self.media_elements.get_mut(element_id) {
            element.tracks.push(track);
            Ok(())
        } else {
            Err("Invalid media element".to_string())
        }
    }
}
