use rustler::NifUnitEnum;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_gathering_state::RTCIceGatheringState;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::signaling_state::RTCSignalingState;

#[derive(NifUnitEnum)]
pub enum IceConnectionState {
    Checking,
    Closed,
    Completed,
    Connected,
    Disconnected,
    Failed,
    New,
    Unspecified,
}

impl From<&RTCIceConnectionState> for IceConnectionState {
    fn from(state: &RTCIceConnectionState) -> Self {
        match state {
            &RTCIceConnectionState::Checking => IceConnectionState::Checking,
            &RTCIceConnectionState::Closed => IceConnectionState::Closed,
            &RTCIceConnectionState::Completed => IceConnectionState::Completed,
            &RTCIceConnectionState::Connected => IceConnectionState::Connected,
            &RTCIceConnectionState::Disconnected => IceConnectionState::Disconnected,
            &RTCIceConnectionState::Failed => IceConnectionState::Failed,
            &RTCIceConnectionState::New => IceConnectionState::New,
            &RTCIceConnectionState::Unspecified => IceConnectionState::Unspecified,
        }
    }
}

#[derive(NifUnitEnum)]
pub enum IceGatheringState {
    Complete,
    Gathering,
    New,
    Unspecified,
}

impl From<&RTCIceGatheringState> for IceGatheringState {
    fn from(state: &RTCIceGatheringState) -> Self {
        match state {
            &RTCIceGatheringState::Complete => IceGatheringState::Complete,
            &RTCIceGatheringState::Gathering => IceGatheringState::Gathering,
            &RTCIceGatheringState::New => IceGatheringState::New,
            &RTCIceGatheringState::Unspecified => IceGatheringState::Unspecified,
        }
    }
}

#[derive(NifUnitEnum)]
pub enum SignalingState {
    Closed,
    HaveLocalOffer,
    HaveLocalPranswer,
    HaveRemoteOffer,
    HaveRemotePranswer,
    Stable,
    Unspecified,
}

impl From<&RTCSignalingState> for SignalingState {
    fn from(state: &RTCSignalingState) -> Self {
        match state {
            &RTCSignalingState::Closed => SignalingState::Closed,
            &RTCSignalingState::HaveLocalOffer => SignalingState::HaveLocalOffer,
            &RTCSignalingState::HaveLocalPranswer => SignalingState::HaveLocalPranswer,
            &RTCSignalingState::HaveRemoteOffer => SignalingState::HaveRemoteOffer,
            &RTCSignalingState::HaveRemotePranswer => SignalingState::HaveRemotePranswer,
            &RTCSignalingState::Stable => SignalingState::Stable,
            &RTCSignalingState::Unspecified => SignalingState::Unspecified,
        }
    }
}

#[derive(NifUnitEnum)]
pub enum ConnectionState {
    Closed,
    Connected,
    Connecting,
    Disconnected,
    Failed,
    New,
    Unspecified,
}

impl From<&RTCPeerConnectionState> for ConnectionState {
    fn from(state: &RTCPeerConnectionState) -> Self {
        match state {
            &RTCPeerConnectionState::Closed => ConnectionState::Closed,
            &RTCPeerConnectionState::Connected => ConnectionState::Connected,
            &RTCPeerConnectionState::Connecting => ConnectionState::Connecting,
            &RTCPeerConnectionState::Disconnected => ConnectionState::Disconnected,
            &RTCPeerConnectionState::Failed => ConnectionState::Failed,
            &RTCPeerConnectionState::New => ConnectionState::New,
            &RTCPeerConnectionState::Unspecified => ConnectionState::Unspecified,
        }
    }
}
