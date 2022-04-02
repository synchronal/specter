defmodule Specter.RtpCodecCapability do
  @moduledoc """
  A representation of webrtc.rs RTCRtpCodecCapability.
  """

  @typedoc """
  For the meaning of specific fields refer to
  https://w3c.github.io/webrtc-pc/#rtcrtpcodeccapability

  Additionaly, webrtc.rs allows to specify extra RTCP packet types.
  However, this is not implemented yet (and also not included in w3c standard).
  """
  @type t() :: %__MODULE__{
          mime_type: String.t(),
          clock_rate: non_neg_integer(),
          channels: non_neg_integer(),
          sdp_fmtp_line: String.t()
        }

  defstruct mime_type: "",
            clock_rate: 0,
            channels: 0,
            sdp_fmtp_line: ""
end
