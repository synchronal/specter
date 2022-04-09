defmodule Specter.TrackLocalStaticSample do
  @moduledoc """
  A representation of webrtc.rs `TrackLocalStaticSample`.

  In general, a track in WebRTC represents a single audio or video
  and its main purpose is to provide user with API for
  sending and receiving media data/packets.

  Therefore, webrtc.rs has multiple implementations of the track depending on
  what user want to do.

  Local tracks are outbound tracks i.e. they are used when user wants to
  send media to the other end of a peer connection.
  User must instantiate local track explicitly.
  At the moment, there are two types of local track: `TrackLocalStaticSample`
  and `TrackLocalStaticRtp`.
  The former is used when user wants RTP encapsulation to be performed under the hood.
  The latter, when user has already prepared RTP packets.

  Remote tracks are inbound tracks i.e. they represent incoming media.
  User does not create remote track explicitly.
  Instead, it announces willingness to receive track by creating a rtp transceiver
  and then, when there are some remote packets, webrtc.rs creates a new
  remote track internally and notifies user.
  """
  alias Specter.Native

  @typedoc """
  Represents an instantiated TrackLocalStaticSample stored in the NIF.
  """
  @opaque t() :: String.t()

  @doc """
  Creates new TrackLocalStaticSample.

  ## Usage

      iex> {:ok, specter} = Specter.init()
      iex> codec = %Specter.RtpCodecCapability{mime_type: "audio"}
      iex> {:ok, _track} = Specter.TrackLocalStaticSample.new(specter, codec, "audio", "specter")
  """
  @spec new(Specter.t(), Specter.RtpCodecCapability.t(), String.t(), String.t()) ::
          {:ok, t()} | {:error, term()}
  def new(%Specter{native: ref}, codec, id, stream_id) do
    Native.new_track_local_static_sample(ref, codec, id, stream_id)
  end

  @doc """
  Reads H264 file and writes it to the track.
  """
  @spec play_from_file_h264(Specter.t(), t(), Path.t()) :: :ok | {:error | term()}
  def play_from_file_h264(%Specter{native: ref}, track, path) do
    if File.exists?(path) do
      Native.play_from_file_h264(ref, track, path)
    else
      {:error, :file_not_found}
    end
  end
end
