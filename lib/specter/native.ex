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
  A media engine with default codecs configured.

  - https://github.com/webrtc-rs/webrtc/blob/master/src/api/media_engine/mod.rs
  """
  @spec new_media_engine(t()) :: {:ok, Specter.media_engine_t()}
  def new_media_engine(_ref), do: error()

  @doc """
  Creates an intercepter registry. This is a user configurable RTP/RTCP pipeline,
  and provides features such as NACKs and RTCP Reports.

  A registry must be created for each peer connection.

  - https://github.com/webrtc-rs/webrtc/blob/master/src/api/interceptor_registry/mod.rs
  """
  @spec new_registry(t(), Specter.media_engine_t()) :: {:ok, Specter.registry_t()}
  def new_registry(_ref, _media_engine), do: error()

  @doc false
  @spec __init__(Specter.init_options()) :: {:ok, t()} | {:error, term()}
  def __init__(_args), do: error()

  defp default_config(args),
    do: Keyword.put_new(args, :ice_servers, Application.get_env(:specter, :default_ice_servers))

  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
