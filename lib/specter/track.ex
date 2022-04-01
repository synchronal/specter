defmodule Specter.TrackLocalStaticSample do
  @moduledoc """
  A representation of webrtc.rs TrackLocalStaticSample.
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
end
