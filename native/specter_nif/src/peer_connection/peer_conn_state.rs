use rustler::NifUnitEnum;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;

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
