defmodule Specter.Native do
  @moduledoc false
  use Rustler, otp_app: :specter, crate: :specter_nif

  @type t() :: reference()

  @doc """
  Initialize the NIF with RTC configuration, registering the current
  process for callbacks.
  """
  @spec init(Specter.init_options()) :: {:ok, t()} | {:error, term()}
  def init(args \\ []) do
    args = default_config(args)
    __init__(Enum.into(args, %{}))
  end

  @doc """
  Given an initialized NIF, get the current config back out into Elixir.

  - https://github.com/webrtc-rs/webrtc/blob/master/src/peer_connection/configuration.rs
  """
  @spec config(t()) :: {:ok, Specter.Config.t()} | {:error, term()}
  def config(_ref), do: error()

  @doc """
  Checks whether the UUID representing a MediaEngine points to an initialized
  MediaEngine that has not been moved into a context owned by some other resource.
  """
  @spec media_engine_exists(t(), Specter.media_engine_t()) :: {:ok, boolean()} | {:error, term()}
  def media_engine_exists(_ref, _media_engine), do: error()

  @doc """
  A media engine with default codecs configured.

  - https://github.com/webrtc-rs/webrtc/blob/master/src/api/media_engine/mod.rs
  """
  @spec new_media_engine(t()) :: {:ok, Specter.media_engine_t()} | {:error, term()}
  def new_media_engine(_ref), do: error()

  @doc """
  An RTCPeerConnection.

  - https://github.com/webrtc-rs/webrtc/blob/master/src/peer_connection/mod.rs
  """
  @spec new_peer_connection(t(), Specter.api_t()) ::
          {:ok, Specter.peer_connection_t()} | {:error, term()}
  def new_peer_connection(_ref, _api), do: error()

  @doc """
  Creates an intercepter registry. This is a user configurable RTP/RTCP pipeline,
  and provides features such as NACKs and RTCP Reports.

  A registry must be created for each peer connection.

  - https://github.com/webrtc-rs/webrtc/blob/master/src/api/interceptor_registry/mod.rs
  """
  @spec new_registry(t(), Specter.media_engine_t()) ::
          {:ok, Specter.registry_t()} | {:error, term()}
  def new_registry(_ref, _media_engine), do: error()

  @doc """
  Creates an API.

  - https://github.com/webrtc-rs/webrtc/blob/master/src/api/mod.rs
  """
  @spec new_api(t(), Specter.media_engine_t(), Specter.registry_t()) ::
          {:ok, Specter.api_t()} | {:error, term()}
  def new_api(_ref, _media_engine, _registry), do: error()

  @doc """
  Checks whether the UUID representing a Registry points to an initialized
  Registry that has not been moved into a context owned by some other resource.
  """
  @spec registry_exists(t(), Specter.registry_t()) :: {:ok, boolean()} | {:error, term()}
  def registry_exists(_ref, _registry), do: error()

  ##
  ## PRIVATE
  ##

  @spec __init__(Specter.init_options()) :: {:ok, t()} | {:error, term()}
  defp __init__(_args), do: error()

  defp default_config(args),
    do: Keyword.put_new(args, :ice_servers, Application.get_env(:specter, :default_ice_servers))

  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
