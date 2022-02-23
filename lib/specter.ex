defmodule Specter do
  @moduledoc """
  Specter is a method for managing data structures and entities provided by
  webrtc.rs. It is intended as a low-level library with some small set of
  opinions, which can composed into more complex behaviors by higher-level
  libraries and applications.

  ## Usage

  A process initializes Specter via the `init/1` function, which registers the
  current process for callbacks that may be triggered via webrtc entities.

      iex> {:ok, _specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])

  ## Thoughts

  During development of the library, it can be assumed that callers will
  implement `handle_info/2` function heads appropriate to the underlying
  implementation. Once these are more solid, it would be nice to `use Specter`,
  which will inject a `handle_info/2` callback, and send the messages to
  other callback functions defined by a behaviour. `handle_ice_candidate`,
  and so on.
  """

  alias Specter.Native

  @opaque t() :: Specter.Native.t()

  @typedoc """
  A STUN uri in the form `stun:host:port`.

  Defaults to `stun:stun.l.google.com:19302`.
  """
  @type stun_server() :: String.t()

  @typedoc """
  Options for initializing RTCPeerConnections. This is set during initialization
  of the library, and later used when creating new connections.
  """
  @type init_options() :: [] | [ice_servers: [stun_server()]]

  @doc """
  Initialize the library. This registers the calling process to receive
  callback messages to `handle_info/2`.
  """
  @spec init() :: {:ok, t()}
  @spec init(init_options()) :: {:ok, t()}
  def init(args \\ []), do: Native.init(args)
end
