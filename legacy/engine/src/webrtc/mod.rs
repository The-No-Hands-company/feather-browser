// WebRTC - Real-time communication

pub struct RTCPeerConnection {
    pub configuration: RTCConfiguration,
    pub local_description: Option<RTCSessionDescription>,
    pub remote_description: Option<RTCSessionDescription>,
    pub signaling_state: SignalingState,
    pub ice_connection_state: ICEConnectionState,
    pub ice_gathering_state: ICEGatheringState,
    pub data_channels: Vec<RTCDataChannel>,
    pub streams: Vec<MediaStream>,
}

#[derive(Debug, Clone)]
pub struct RTCConfiguration {
    pub ice_servers: Vec<RTCIceServer>,
    pub ice_transport_policy: ICETransportPolicy,
    pub bundle_policy: BundlePolicy,
}

#[derive(Debug, Clone)]
pub struct RTCIceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ICETransportPolicy {
    All,
    Relay,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BundlePolicy {
    Balanced,
    MaxCompat,
    MaxBundle,
}

#[derive(Debug, Clone)]
pub struct RTCSessionDescription {
    pub sdp_type: SDPType,
    pub sdp: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SDPType {
    Offer,
    Answer,
    Pranswer,
    Rollback,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignalingState {
    Stable,
    HaveLocalOffer,
    HaveRemoteOffer,
    HaveLocalPranswer,
    HaveRemotePranswer,
    Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ICEConnectionState {
    New,
    Checking,
    Connected,
    Completed,
    Failed,
    Disconnected,
    Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ICEGatheringState {
    New,
    Gathering,
    Complete,
}

#[derive(Debug, Clone)]
pub struct RTCDataChannel {
    pub label: String,
    pub ordered: bool,
    pub max_packet_life_time: Option<u32>,
    pub max_retransmits: Option<u32>,
    pub protocol: String,
    pub negotiated: bool,
    pub id: Option<u16>,
    pub ready_state: DataChannelState,
    pub buffered_amount: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataChannelState {
    Connecting,
    Open,
    Closing,
    Closed,
}

#[derive(Debug, Clone)]
pub struct MediaStream {
    pub id: String,
    pub tracks: Vec<MediaStreamTrack>,
}

#[derive(Debug, Clone)]
pub struct MediaStreamTrack {
    pub kind: MediaStreamTrackKind,
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub muted: bool,
    pub ready_state: MediaStreamTrackState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaStreamTrackKind {
    Audio,
    Video,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaStreamTrackState {
    Live,
    Ended,
}

impl RTCPeerConnection {
    pub fn new(configuration: RTCConfiguration) -> Self {
        RTCPeerConnection {
            configuration,
            local_description: None,
            remote_description: None,
            signaling_state: SignalingState::Stable,
            ice_connection_state: ICEConnectionState::New,
            ice_gathering_state: ICEGatheringState::New,
            data_channels: Vec::new(),
            streams: Vec::new(),
        }
    }

    pub fn create_offer(&mut self) -> Result<RTCSessionDescription, String> {
        if self.signaling_state != SignalingState::Stable {
            return Err("Invalid signaling state".to_string());
        }

        let offer = RTCSessionDescription {
            sdp_type: SDPType::Offer,
            sdp: "v=0\no=- 0 0 IN IP4 127.0.0.1\ns=-\nt=0 0\n".to_string(),
        };

        self.signaling_state = SignalingState::HaveLocalOffer;
        Ok(offer)
    }

    pub fn create_answer(&mut self) -> Result<RTCSessionDescription, String> {
        if self.signaling_state != SignalingState::HaveRemoteOffer {
            return Err("Invalid signaling state".to_string());
        }

        let answer = RTCSessionDescription {
            sdp_type: SDPType::Answer,
            sdp: "v=0\no=- 0 0 IN IP4 127.0.0.1\ns=-\nt=0 0\n".to_string(),
        };

        self.signaling_state = SignalingState::Stable;
        Ok(answer)
    }

    pub fn set_local_description(&mut self, description: RTCSessionDescription) {
        self.local_description = Some(description);
    }

    pub fn set_remote_description(&mut self, description: RTCSessionDescription) {
        let sdp_type = description.sdp_type.clone();
        self.remote_description = Some(description);
        if sdp_type == SDPType::Offer {
            self.signaling_state = SignalingState::HaveRemoteOffer;
        }
    }

    pub fn create_data_channel(&mut self, label: &str, options: DataChannelOptions) -> usize {
        let channel = RTCDataChannel {
            label: label.to_string(),
            ordered: options.ordered,
            max_packet_life_time: options.max_packet_life_time,
            max_retransmits: options.max_retransmits,
            protocol: options.protocol,
            negotiated: options.negotiated,
            id: options.id,
            ready_state: DataChannelState::Connecting,
            buffered_amount: 0,
        };
        self.data_channels.push(channel);
        self.data_channels.len() - 1
    }

    pub fn add_stream(&mut self, stream: MediaStream) {
        self.streams.push(stream);
    }

    pub fn close(&mut self) {
        self.signaling_state = SignalingState::Closed;
        self.ice_connection_state = ICEConnectionState::Closed;
    }
}

#[derive(Debug, Clone)]
pub struct DataChannelOptions {
    pub ordered: bool,
    pub max_packet_life_time: Option<u32>,
    pub max_retransmits: Option<u32>,
    pub protocol: String,
    pub negotiated: bool,
    pub id: Option<u16>,
}

impl Default for DataChannelOptions {
    fn default() -> Self {
        DataChannelOptions {
            ordered: true,
            max_packet_life_time: None,
            max_retransmits: None,
            protocol: String::new(),
            negotiated: false,
            id: None,
        }
    }
}
