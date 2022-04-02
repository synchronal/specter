use rustler::NifStruct;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;

#[derive(NifStruct)]
#[module = "Specter.RtpCodecCapability"]
pub struct RtpCodecCapability {
    pub mime_type: String,
    pub clock_rate: u32,
    pub channels: u16,
    pub sdp_fmtp_line: String,
}

impl From<&RtpCodecCapability> for RTCRtpCodecCapability {
    fn from(rtp_codec_capability: &RtpCodecCapability) -> Self {
        RTCRtpCodecCapability {
            mime_type: rtp_codec_capability.mime_type.clone(),
            clock_rate: rtp_codec_capability.clock_rate,
            channels: rtp_codec_capability.channels,
            sdp_fmtp_line: rtp_codec_capability.sdp_fmtp_line.clone(),
            ..Default::default()
        }
    }
}

impl From<RtpCodecCapability> for RTCRtpCodecCapability {
    fn from(rtp_codec_capability: RtpCodecCapability) -> Self {
        RTCRtpCodecCapability {
            mime_type: rtp_codec_capability.mime_type,
            clock_rate: rtp_codec_capability.clock_rate,
            channels: rtp_codec_capability.channels,
            sdp_fmtp_line: rtp_codec_capability.sdp_fmtp_line,
            ..Default::default()
        }
    }
}
