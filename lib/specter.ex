defmodule Specter do
  @moduledoc """
  Specter is a method for managing data structures and entities provided by
  webrtc.rs. It is intended as a low-level library with some small set of
  opinions, which can composed into more complex behaviors by higher-level
  libraries and applications.

  ## Usage

  A process initializes Specter via the `init/1` function, which registers the
  current process for callbacks that may be triggered via webrtc entities.

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> {:ok, _uuid} = Specter.new_media_engine(specter)

  ## Thoughts

  During development of the library, it can be assumed that callers will
  implement `handle_info/2` function heads appropriate to the underlying
  implementation. Once these are more solid, it would be nice to `use Specter`,
  which will inject a `handle_info/2` callback, and send the messages to
  other callback functions defined by a behaviour. `handle_ice_candidate`,
  and so on.
  """

  alias Specter.Native

  @typedoc """
  `t:Specter.t/0` references are returned from `init/1`, and represent the
  state held in the NIF. All functions interacting with NIF state take a
  `t:Specter.t/0` as their first argument.
  """
  @opaque t() :: Specter.Native.t()

  @typedoc """
  A STUN uri in the form `protocol:host:port`, where protocol is either
  `stun` or `turn`.

  Defaults to `stun:stun.l.google.com:19302`.
  """
  @type ice_server() :: String.t()

  @typedoc """
  Options for initializing RTCPeerConnections. This is set during initialization
  of the library, and later used when creating new connections.
  """
  @type init_options() :: [] | [ice_servers: [ice_server()]]

  @typedoc """
  While entities managed by Specter are generally accessed via the
  `t:Specter.t/0` returned from `init/1`, specific entities are accessed
  via UUIDs returned from the functions used to initialize them (e.g.
  `new_rtc_peer_connection/1`).
  """
  @type uuid() :: String.t()

  @doc """
  Initialize the library. This registers the calling process to receive
  callback messages to `handle_info/2`.

  | param         | type               | default |
  | ------------- | ------------------ | ------- |
  | `ice_servers` | `list(String.t())` | `["stun:stun.l.google.com:19302"]` |
  """
  @spec init() :: {:ok, t()}
  @spec init(init_options()) :: {:ok, t()} | {:error, term()}
  def init(args \\ []), do: Native.init(args)

  @doc """
  Returns the currently
  """
  @spec config(t()) :: {:ok, Specter.Config.t()} | {:error, term()}
  def config(ref), do: Native.config(ref)

  @spec new_media_engine(t()) :: {:ok, uuid()}
  def new_media_engine(ref), do: Native.new_media_engine(ref)
end
