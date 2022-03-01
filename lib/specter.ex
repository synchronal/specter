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
      ...>
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      ...>
      iex> Specter.media_engine_exists?(specter, media_engine)
      true
      iex> Specter.registry_exists?(specter, registry)
      true
      ...>
      iex> {:ok, _api} = Specter.new_api(specter, media_engine, registry)
      ...>
      iex> Specter.media_engine_exists?(specter, media_engine)
      false
      iex> Specter.registry_exists?(specter, registry)
      false

  ## Thoughts

  During development of the library, it can be assumed that callers will
  implement `handle_info/2` function heads appropriate to the underlying
  implementation. Once these are more solid, it would be nice to `use Specter`,
  which will inject a `handle_info/2` callback, and send the messages to
  other callback functions defined by a behaviour. `handle_ice_candidate`,
  and so on.

  Some things are returned from the NIF as UUIDs. These are declared as `@opaque`,
  to indicate that users of the library should not rely of them being in a
  particular format. They could change later to be references, for instance.
  """

  alias Specter.Native

  @typedoc """
  `t:Specter.t/0` references are returned from `init/1`, and represent the
  state held in the NIF. All functions interacting with NIF state take a
  `t:Specter.t/0` as their first argument.
  """
  @opaque t() :: Specter.Native.t()

  @typedoc """
  `t:Specter.media_engine_t/0` represents an instantiated MediaEngine managed in the NIF.
  """
  @opaque media_engine_t() :: String.t()

  @typedoc """
  `t:Specter.registry_t/0` represent an instantiated intercepter Registry managed in the NIF.
  """
  @opaque registry_t() :: String.t()

  @typedoc """
  `t:Specter.api_t/0` represent an instantiated APIBuilder managed in the NIF.
  """
  @opaque api_t() :: String.t()

  @typedoc """
  A uri in the form `protocol:host:port`, where protocol is either
  `stun` or `turn`.

  Defaults to `stun:stun.l.google.com:19302`.
  """
  @type ice_server() :: String.t()

  @typedoc """
  Options for initializing RTCPeerConnections. This is set during initialization
  of the library, and later used when creating new connections.
  """
  @type init_options() :: [] | [ice_servers: [ice_server()]]

  @doc """
  Initialize the library. This registers the calling process to receive
  callback messages to `handle_info/2`.

  | param         | type               | default |
  | ------------- | ------------------ | ------- |
  | `ice_servers` | `list(String.t())` | `["stun:stun.l.google.com:19302"]` |

  ## Usage

      iex> {:ok, _specter} = Specter.init(ice_servers: ["stun:stun.example.com:3478"])

  """
  @spec init() :: {:ok, t()}
  @spec init(init_options()) :: {:ok, t()} | {:error, term()}
  def init(args \\ []), do: Native.init(args)

  @doc """
  Returns the current configuration for the initialized NIF.

  ## Usage

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.example.com:3478"])
      iex> Specter.config(specter)
      {:ok, %Specter.Config{ice_servers: ["stun:stun.example.com:3478"]}}

  """
  @spec config(t()) :: {:ok, Specter.Config.t()} | {:error, term()}
  def config(ref), do: Native.config(ref)

  @doc """
  Returns true or false, depending on whether the media engine is available for
  consumption, i.e. is initialized and has not been used by a function that takes
  ownership of it.

  ## Usage

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> Specter.media_engine_exists?(specter, media_engine)
      true

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> Specter.media_engine_exists?(specter, UUID.uuid4())
      false

  """
  @spec media_engine_exists?(t(), media_engine_t()) :: boolean() | no_return()
  def media_engine_exists?(ref, media_engine) do
    case Native.media_engine_exists(ref, media_engine) do
      {:ok, value} ->
        value

      {:error, error} ->
        raise "Unable to determine whether media engine exists:\n#{inspect(error)}"
    end
  end

  @doc """
  An APIBuilder is used to create RTCPeerConnections. This accepts as parameters
  the output of `init/1`, `new_media_enine/1`, and `new_registry/2`.

  Note that this takes ownership of both the media engine and the registry,
  effectively consuming them.

  | param          | type     | default |
  | -------------- | -------- | ------- |
  | `specter`      | `t()`    | |
  | `media_engine` | `opaque` | |
  | `registry`     | `opaque` | |

  ## Usage

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, _api} = Specter.new_api(specter, media_engine, registry)

  """
  @spec new_api(t(), media_engine_t(), registry_t()) :: {:ok, api_t()}
  def new_api(ref, media_engine, registry),
    do: Native.new_api(ref, media_engine, registry)

  @doc """
  Creates a MediaEngine to be configured and used by later function calls.
  Codecs and other high level configuration are done on instances of MediaEngines.
  A MediaEngine is combined with a Registry in an entity called an APIBuilder,
  which is then used to create RTCPeerConnections.

  ## Usage

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> {:ok, _media_engine} = Specter.new_media_engine(specter)

  """
  @spec new_media_engine(t()) :: {:ok, media_engine_t()}
  def new_media_engine(ref), do: Native.new_media_engine(ref)

  @doc """
  Creates an intercepter registry. This is a user configurable RTP/RTCP pipeline,
  and provides features such as NACKs and RTCP Reports.

  The registry may be combined with a MediaEngine in an APIBuilder, which is then used
  to create RTCPeerConnections.

  Note that creating a registry does **not** take ownership of the media engine.

  ## Usage

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, _registry} = Specter.new_registry(specter, media_engine)
      ...>
      iex> Specter.media_engine_exists?(specter, media_engine)
      true

  """
  @spec new_registry(t(), media_engine_t()) :: {:ok, registry_t()}
  def new_registry(ref, media_engine), do: Native.new_registry(ref, media_engine)

  @doc """
  Returns true or false, depending on whether the registry is available for
  consumption, i.e. is initialized and has not been used by a function that takes
  ownership of it.

  ## Usage

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> Specter.registry_exists?(specter, registry)
      true

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> Specter.registry_exists?(specter, UUID.uuid4())
      false
  """
  @spec registry_exists?(t(), registry_t()) :: boolean() | no_return()
  def registry_exists?(ref, registry) do
    case Native.registry_exists(ref, registry) do
      {:ok, value} ->
        value

      {:error, error} ->
        raise "Unable to determine whether registry exists:\n#{inspect(error)}"
    end
  end
end
