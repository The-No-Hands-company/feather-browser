// Web Audio API Implementation
use std::collections::HashMap;

pub struct AudioContext {
    sample_rate: f32,
    current_time: f64,
    destination: AudioDestinationNode,
    nodes: HashMap<usize, Box<dyn AudioNode>>,
    next_id: usize,
}

impl AudioContext {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0,
            current_time: 0.0,
            destination: AudioDestinationNode::new(),
            nodes: HashMap::new(),
            next_id: 1,
        }
    }
    
    pub fn create_oscillator(&mut self) -> OscillatorNode {
        let node = OscillatorNode::new(self.sample_rate);
        node
    }
    
    pub fn create_gain(&mut self) -> GainNode {
        GainNode::new()
    }
    
    pub fn create_buffer_source(&mut self) -> AudioBufferSourceNode {
        AudioBufferSourceNode::new()
    }
    
    pub fn create_analyser(&mut self) -> AnalyserNode {
        AnalyserNode::new()
    }
    
    pub fn create_biquad_filter(&mut self) -> BiquadFilterNode {
        BiquadFilterNode::new()
    }
    
    pub fn create_convolver(&mut self) -> ConvolverNode {
        ConvolverNode::new()
    }
    
    pub fn create_delay(&mut self, max_delay_time: f64) -> DelayNode {
        DelayNode::new(max_delay_time)
    }
    
    pub fn create_dynamics_compressor(&mut self) -> DynamicsCompressorNode {
        DynamicsCompressorNode::new()
    }
    
    pub fn create_panner(&mut self) -> PannerNode {
        PannerNode::new()
    }
    
    pub fn create_stereo_panner(&mut self) -> StereoPannerNode {
        StereoPannerNode::new()
    }
    
    pub fn create_wave_shaper(&mut self) -> WaveShaperNode {
        WaveShaperNode::new()
    }
    
    pub fn decode_audio_data(&self, _data: &[u8]) -> Result<AudioBuffer, String> {
        // TODO: Decode audio data (MP3, WAV, OGG, etc.) using appropriate codec
        // For now, return empty buffer with standard CD quality specs
        Ok(AudioBuffer::new(2, 44100, 44100.0))
    }
}

pub trait AudioNode {
    fn connect(&mut self, destination: &dyn AudioNode);
    fn disconnect(&mut self);
}

#[derive(Debug, Clone)]
pub struct AudioDestinationNode {
    max_channel_count: usize,
}

impl AudioDestinationNode {
    pub fn new() -> Self {
        Self {
            max_channel_count: 2,
        }
    }
}

impl AudioNode for AudioDestinationNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct OscillatorNode {
    frequency: AudioParam,
    detune: AudioParam,
    oscillator_type: OscillatorType,
    started: bool,
}

#[derive(Debug, Clone)]
pub enum OscillatorType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Custom,
}

impl OscillatorNode {
    pub fn new(_sample_rate: f32) -> Self {
        Self {
            frequency: AudioParam::new(440.0),
            detune: AudioParam::new(0.0),
            oscillator_type: OscillatorType::Sine,
            started: false,
        }
    }
    
    pub fn set_type(&mut self, osc_type: OscillatorType) {
        self.oscillator_type = osc_type;
    }
    
    pub fn get_frequency(&self) -> &AudioParam {
        &self.frequency
    }
    
    pub fn get_frequency_mut(&mut self) -> &mut AudioParam {
        &mut self.frequency
    }
    
    pub fn get_detune(&self) -> &AudioParam {
        &self.detune
    }
    
    pub fn get_detune_mut(&mut self) -> &mut AudioParam {
        &mut self.detune
    }
    
    pub fn generate_sample(&self, time: f64) -> f32 {
        let freq = self.frequency.value + (self.detune.value / 1200.0) * self.frequency.value;
        let phase = 2.0 * std::f64::consts::PI * freq as f64 * time;
        
        match self.oscillator_type {
            OscillatorType::Sine => phase.sin() as f32,
            OscillatorType::Square => if phase.sin() >= 0.0 { 1.0 } else { -1.0 },
            OscillatorType::Sawtooth => (2.0 * (phase / (2.0 * std::f64::consts::PI) - (phase / (2.0 * std::f64::consts::PI) + 0.5).floor())) as f32,
            OscillatorType::Triangle => (4.0 * ((phase / (2.0 * std::f64::consts::PI)).fract() - 0.5).abs() - 1.0) as f32,
            OscillatorType::Custom => 0.0,
        }
    }
    
    pub fn start(&mut self, when: f64) {
        if when >= 0.0 {
            self.started = true;
        }
    }
    
    pub fn stop(&mut self, when: f64) {
        if when >= 0.0 {
            self.started = false;
        }
    }
}

impl AudioNode for OscillatorNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct GainNode {
    gain: AudioParam,
}

impl GainNode {
    pub fn new() -> Self {
        Self {
            gain: AudioParam::new(1.0),
        }
    }
    
    pub fn get_gain(&self) -> &AudioParam {
        &self.gain
    }
    
    pub fn get_gain_mut(&mut self) -> &mut AudioParam {
        &mut self.gain
    }
    
    pub fn apply_gain(&self, sample: f32) -> f32 {
        sample * self.gain.value
    }
}

impl AudioNode for GainNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct AudioParam {
    value: f32,
    default_value: f32,
    min_value: f32,
    max_value: f32,
}

impl AudioParam {
    pub fn new(default_value: f32) -> Self {
        Self {
            value: default_value,
            default_value,
            min_value: f32::MIN,
            max_value: f32::MAX,
        }
    }
    
    pub fn get_value(&self) -> f32 {
        self.value
    }
    
    pub fn get_default_value(&self) -> f32 {
        self.default_value
    }
    
    pub fn reset_to_default(&mut self) {
        self.value = self.default_value;
    }
    
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min_value, self.max_value);
    }
    
    pub fn set_value_at_time(&mut self, value: f32, _start_time: f64) {
        self.value = value;
    }
    
    pub fn linear_ramp_to_value_at_time(&mut self, value: f32, _end_time: f64) {
        self.value = value;
    }
    
    pub fn exponential_ramp_to_value_at_time(&mut self, value: f32, _end_time: f64) {
        self.value = value;
    }
}

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    number_of_channels: usize,
    length: usize,
    sample_rate: f32,
    data: Vec<Vec<f32>>,
}

impl AudioBuffer {
    pub fn new(number_of_channels: usize, length: usize, sample_rate: f32) -> Self {
        let mut data = Vec::new();
        for _ in 0..number_of_channels {
            data.push(vec![0.0; length]);
        }
        
        Self {
            number_of_channels,
            length,
            sample_rate,
            data,
        }
    }
    
    pub fn get_number_of_channels(&self) -> usize {
        self.number_of_channels
    }
    
    pub fn get_length(&self) -> usize {
        self.length
    }
    
    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }
    
    pub fn get_duration(&self) -> f64 {
        self.length as f64 / self.sample_rate as f64
    }
    
    pub fn get_channel_data(&self, channel: usize) -> Option<&[f32]> {
        self.data.get(channel).map(|v| v.as_slice())
    }
    
    pub fn copy_to_channel(&mut self, source: &[f32], channel: usize) {
        if let Some(dest) = self.data.get_mut(channel) {
            let len = source.len().min(dest.len());
            dest[..len].copy_from_slice(&source[..len]);
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioBufferSourceNode {
    buffer: Option<AudioBuffer>,
    loop_enabled: bool,
    loop_start: f64,
    loop_end: f64,
    playback_rate: AudioParam,
}

impl AudioBufferSourceNode {
    pub fn new() -> Self {
        Self {
            buffer: None,
            loop_enabled: false,
            loop_start: 0.0,
            loop_end: 0.0,
            playback_rate: AudioParam::new(1.0),
        }
    }
    
    pub fn set_buffer(&mut self, buffer: AudioBuffer) {
        self.buffer = Some(buffer);
    }
    
    pub fn get_buffer(&self) -> Option<&AudioBuffer> {
        self.buffer.as_ref()
    }
    
    pub fn set_loop(&mut self, enabled: bool) {
        self.loop_enabled = enabled;
    }
    
    pub fn set_loop_start(&mut self, start: f64) {
        self.loop_start = start;
    }
    
    pub fn set_loop_end(&mut self, end: f64) {
        self.loop_end = end;
    }
    
    pub fn get_playback_rate(&self) -> &AudioParam {
        &self.playback_rate
    }
    
    pub fn get_playback_rate_mut(&mut self) -> &mut AudioParam {
        &mut self.playback_rate
    }
    
    pub fn start(&mut self, when: f64) {
        if when >= 0.0 && self.buffer.is_some() {
            // Schedule with audio rendering thread
        }
    }
    
    pub fn stop(&mut self, when: f64) {
        if when >= 0.0 {
            // Schedule stop with audio rendering thread
        }
    }
}

impl AudioNode for AudioBufferSourceNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct AnalyserNode {
    fft_size: usize,
    frequency_bin_count: usize,
    min_decibels: f32,
    max_decibels: f32,
    smoothing_time_constant: f32,
}

impl AnalyserNode {
    pub fn new() -> Self {
        Self {
            fft_size: 2048,
            frequency_bin_count: 1024,
            min_decibels: -100.0,
            max_decibels: -30.0,
            smoothing_time_constant: 0.8,
        }
    }
    
    pub fn set_fft_size(&mut self, size: usize) {
        self.fft_size = size;
        self.frequency_bin_count = size / 2;
    }
    
    pub fn get_fft_size(&self) -> usize {
        self.fft_size
    }
    
    pub fn set_min_decibels(&mut self, db: f32) {
        self.min_decibels = db;
    }
    
    pub fn set_max_decibels(&mut self, db: f32) {
        self.max_decibels = db;
    }
    
    pub fn set_smoothing_time_constant(&mut self, constant: f32) {
        self.smoothing_time_constant = constant.clamp(0.0, 1.0);
    }
    
    pub fn get_byte_frequency_data(&self) -> Vec<u8> {
        vec![0; self.frequency_bin_count]
    }
    
    pub fn get_byte_time_domain_data(&self) -> Vec<u8> {
        vec![128; self.fft_size]
    }
    
    pub fn get_float_frequency_data(&self) -> Vec<f32> {
        vec![0.0; self.frequency_bin_count]
    }
    
    pub fn get_float_time_domain_data(&self) -> Vec<f32> {
        vec![0.0; self.fft_size]
    }
}

impl AudioNode for AnalyserNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct BiquadFilterNode {
    filter_type: BiquadFilterType,
    frequency: AudioParam,
    q: AudioParam,
    gain: AudioParam,
}

#[derive(Debug, Clone)]
pub enum BiquadFilterType {
    Lowpass,
    Highpass,
    Bandpass,
    Lowshelf,
    Highshelf,
    Peaking,
    Notch,
    Allpass,
}

impl BiquadFilterNode {
    pub fn new() -> Self {
        Self {
            filter_type: BiquadFilterType::Lowpass,
            frequency: AudioParam::new(350.0),
            q: AudioParam::new(1.0),
            gain: AudioParam::new(0.0),
        }
    }
    
    pub fn set_type(&mut self, filter_type: BiquadFilterType) {
        self.filter_type = filter_type;
    }
    
    pub fn get_type(&self) -> &BiquadFilterType {
        &self.filter_type
    }
    
    pub fn get_frequency(&self) -> &AudioParam {
        &self.frequency
    }
    
    pub fn get_frequency_mut(&mut self) -> &mut AudioParam {
        &mut self.frequency
    }
    
    pub fn get_q(&self) -> &AudioParam {
        &self.q
    }
    
    pub fn get_q_mut(&mut self) -> &mut AudioParam {
        &mut self.q
    }
    
    pub fn get_gain(&self) -> &AudioParam {
        &self.gain
    }
    
    pub fn get_gain_mut(&mut self) -> &mut AudioParam {
        &mut self.gain
    }
}

impl AudioNode for BiquadFilterNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct ConvolverNode {
    buffer: Option<AudioBuffer>,
    normalize: bool,
}

impl ConvolverNode {
    pub fn new() -> Self {
        Self {
            buffer: None,
            normalize: true,
        }
    }
    
    pub fn set_buffer(&mut self, buffer: AudioBuffer) {
        self.buffer = Some(buffer);
    }
    
    pub fn get_buffer(&self) -> Option<&AudioBuffer> {
        self.buffer.as_ref()
    }
    
    pub fn set_normalize(&mut self, normalize: bool) {
        self.normalize = normalize;
    }
    
    pub fn get_normalize(&self) -> bool {
        self.normalize
    }
}

impl AudioNode for ConvolverNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct DelayNode {
    delay_time: AudioParam,
    max_delay_time: f64,
}

impl DelayNode {
    pub fn new(max_delay_time: f64) -> Self {
        Self {
            delay_time: AudioParam::new(0.0),
            max_delay_time,
        }
    }
    
    pub fn get_delay_time(&self) -> &AudioParam {
        &self.delay_time
    }
    
    pub fn get_delay_time_mut(&mut self) -> &mut AudioParam {
        &mut self.delay_time
    }
    
    pub fn get_max_delay_time(&self) -> f64 {
        self.max_delay_time
    }
}

impl AudioNode for DelayNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct DynamicsCompressorNode {
    threshold: AudioParam,
    knee: AudioParam,
    ratio: AudioParam,
    attack: AudioParam,
    release: AudioParam,
}

impl DynamicsCompressorNode {
    pub fn new() -> Self {
        Self {
            threshold: AudioParam::new(-24.0),
            knee: AudioParam::new(30.0),
            ratio: AudioParam::new(12.0),
            attack: AudioParam::new(0.003),
            release: AudioParam::new(0.25),
        }
    }
    
    pub fn get_threshold(&self) -> &AudioParam {
        &self.threshold
    }
    
    pub fn get_threshold_mut(&mut self) -> &mut AudioParam {
        &mut self.threshold
    }
    
    pub fn get_knee(&self) -> &AudioParam {
        &self.knee
    }
    
    pub fn get_knee_mut(&mut self) -> &mut AudioParam {
        &mut self.knee
    }
    
    pub fn get_ratio(&self) -> &AudioParam {
        &self.ratio
    }
    
    pub fn get_ratio_mut(&mut self) -> &mut AudioParam {
        &mut self.ratio
    }
    
    pub fn get_attack(&self) -> &AudioParam {
        &self.attack
    }
    
    pub fn get_attack_mut(&mut self) -> &mut AudioParam {
        &mut self.attack
    }
    
    pub fn get_release(&self) -> &AudioParam {
        &self.release
    }
    
    pub fn get_release_mut(&mut self) -> &mut AudioParam {
        &mut self.release
    }
}

impl AudioNode for DynamicsCompressorNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct PannerNode {
    position_x: AudioParam,
    position_y: AudioParam,
    position_z: AudioParam,
    orientation_x: AudioParam,
    orientation_y: AudioParam,
    orientation_z: AudioParam,
}

impl PannerNode {
    pub fn new() -> Self {
        Self {
            position_x: AudioParam::new(0.0),
            position_y: AudioParam::new(0.0),
            position_z: AudioParam::new(0.0),
            orientation_x: AudioParam::new(1.0),
            orientation_y: AudioParam::new(0.0),
            orientation_z: AudioParam::new(0.0),
        }
    }
    
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position_x.set_value(x);
        self.position_y.set_value(y);
        self.position_z.set_value(z);
    }
    
    pub fn set_orientation(&mut self, x: f32, y: f32, z: f32) {
        self.orientation_x.set_value(x);
        self.orientation_y.set_value(y);
        self.orientation_z.set_value(z);
    }
    
    pub fn get_position_x(&self) -> &AudioParam {
        &self.position_x
    }
    
    pub fn get_position_y(&self) -> &AudioParam {
        &self.position_y
    }
    
    pub fn get_position_z(&self) -> &AudioParam {
        &self.position_z
    }
    
    pub fn get_orientation_x(&self) -> &AudioParam {
        &self.orientation_x
    }
    
    pub fn get_orientation_y(&self) -> &AudioParam {
        &self.orientation_y
    }
    
    pub fn get_orientation_z(&self) -> &AudioParam {
        &self.orientation_z
    }
}

impl AudioNode for PannerNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct StereoPannerNode {
    pan: AudioParam,
}

impl StereoPannerNode {
    pub fn new() -> Self {
        Self {
            pan: AudioParam::new(0.0),
        }
    }
    
    pub fn get_pan(&self) -> &AudioParam {
        &self.pan
    }
    
    pub fn get_pan_mut(&mut self) -> &mut AudioParam {
        &mut self.pan
    }
    
    pub fn apply_pan(&self, left: f32, right: f32) -> (f32, f32) {
        let pan_value = self.pan.value.clamp(-1.0, 1.0);
        if pan_value <= 0.0 {
            (left, right * (1.0 + pan_value))
        } else {
            (left * (1.0 - pan_value), right)
        }
    }
}

impl AudioNode for StereoPannerNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct WaveShaperNode {
    curve: Option<Vec<f32>>,
    oversample: OversampleType,
}

#[derive(Debug, Clone)]
pub enum OversampleType {
    None,
    TwoX,
    FourX,
}

impl WaveShaperNode {
    pub fn new() -> Self {
        Self {
            curve: None,
            oversample: OversampleType::None,
        }
    }
    
    pub fn set_curve(&mut self, curve: Vec<f32>) {
        self.curve = Some(curve);
    }
    
    pub fn get_curve(&self) -> Option<&[f32]> {
        self.curve.as_ref().map(|c| c.as_slice())
    }
    
    pub fn set_oversample(&mut self, oversample: OversampleType) {
        self.oversample = oversample;
    }
    
    pub fn get_oversample(&self) -> &OversampleType {
        &self.oversample
    }
    
    pub fn apply_curve(&self, input: f32) -> f32 {
        if let Some(curve) = &self.curve {
            let index = ((input + 1.0) / 2.0 * (curve.len() - 1) as f32) as usize;
            curve.get(index).copied().unwrap_or(input)
        } else {
            input
        }
    }
}

impl AudioNode for WaveShaperNode {
    fn connect(&mut self, _destination: &dyn AudioNode) {}
    fn disconnect(&mut self) {}
}
