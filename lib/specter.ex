defmodule Specter do
  @moduledoc """
  Documentation for `Specter`.
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

  @spec init() :: {:ok, t()}
  @spec init(init_options()) :: {:ok, t()}
  def init(args \\ []), do: Native.init(args)
end
