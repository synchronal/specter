defmodule Specter do
  @moduledoc """
  Specter is a method for managing data structures and entities provided by
  `webrtc.rs`. It is intended as a low-level library with some small set of
  opinions, which can composed into more complex behaviors by higher-level
  libraries and applications.

  ## Key points

  Specter wraps `webrtc.rs`, which heavily utilizes async Rust. For this reason,
  many functions cannot be automatically awaited by the caller—the NIF functions
  send messages across channels to separate threads managed by Rust, which send
  messages back to Elixir that can be caught by `receive` or `handle_info`.

  ## Usage

  A process initializes Specter via the `init/1` function, which registers the
  current process for callbacks that may be triggered via webrtc entities.

      iex> ## Initialize the library. Register messages to the current pid.
      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      ...>
      iex> ## Create a peer connection's dependencies
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      ...>
      iex> Specter.media_engine_exists?(specter, media_engine)
      true
      iex> Specter.registry_exists?(specter, registry)
      true
      ...>
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      ...>
      iex> Specter.media_engine_exists?(specter, media_engine)
      false
      iex> Specter.registry_exists?(specter, registry)
      false
      ...>
      iex> ## Create a peer connection
      iex> {:ok, pc_1} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc_1}
      iex> Specter.peer_connection_exists?(specter, pc_1)
      true
      iex> ## Add a thing to be negotiated
      iex> :ok = Specter.create_data_channel(specter, pc_1, "data")
      iex> assert_receive {:data_channel_created, ^pc_1}
      ...>
      iex> ## Create an offer
      iex> :ok = Specter.create_offer(specter, pc_1)
      iex> assert_receive {:offer, ^pc_1, offer}
      iex> :ok = Specter.set_local_description(specter, pc_1, offer)
      iex> assert_receive {:ok, ^pc_1, :set_local_description}
      ...>
      iex> ## Create a second peer connection, to answer back
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc_2} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc_2}
      ...>
      iex> ## Begin negotiating offer/answer
      iex> :ok = Specter.set_remote_description(specter, pc_2, offer)
      iex> assert_receive {:ok, ^pc_2, :set_remote_description}
      iex> :ok = Specter.create_answer(specter, pc_2)
      iex> assert_receive {:answer, ^pc_2, answer}
      iex> :ok = Specter.set_local_description(specter, pc_2, answer)
      iex> assert_receive {:ok, ^pc_2, :set_local_description}
      ...>
      iex> ## Receive ice candidates
      iex> assert_receive {:ice_candidate, ^pc_1, _candidate}
      iex> assert_receive {:ice_candidate, ^pc_2, _candidate}
      ...>
      iex> ## Shut everything down
      iex> Specter.close_peer_connection(specter, pc_1)
      :ok
      iex> assert_receive {:peer_connection_closed, ^pc_1}
      ...>
      iex> :ok = Specter.close_peer_connection(specter, pc_2)
      iex> assert_receive {:peer_connection_closed, ^pc_2}

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

  @enforce_keys [:native]
  defstruct [:native]

  @typedoc """
  `t:native_t/0` references are returned from the NIF, and represent state held
  in Rust code.
  """
  @opaque native_t() :: Specter.Native.t()

  @typedoc """
  `t:Specter.t/0` wraps the reference returned from `init/1`. All functions interacting with
  NIF state take a `t:Specter.t/0` as their first argument.
  """
  @type t() :: %Specter{native: native_t()}

  @typedoc """
  `t:Specter.api_t/0` represent an instantiated API managed in the NIF.
  """
  @opaque api_t() :: String.t()

  @typedoc """
  `t:Specter.media_engine_t/0` represents an instantiated MediaEngine managed in the NIF.
  """
  @opaque media_engine_t() :: String.t()

  @typedoc """
  `t:Specter.peer_connection_t/0` represents an instantiated RTCPeerConnection managed in the NIF.
  """
  @opaque peer_connection_t() :: String.t()

  @typedoc """
  `t:Specter.registry_t/0` represent an instantiated intercepter Registry managed in the NIF.
  """
  @opaque registry_t() :: String.t()

  @typedoc """
  An ICE candidate as JSON.
  """
  @type ice_candidate_t() :: String.t()

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

  @typedoc """
  Options for creating a webrtc answer. Values default to false.
  """
  @type answer_options_t() :: [] | [voice_activity_detection: bool]

  @typedoc """
  Options for creating a webrtc offer. Values default to false.
  """
  @type offer_options_t() :: [] | [voice_activity_detection: bool, ice_restart: bool]

  @typedoc """
  The type of an SDP message, either an `:offer` or an `:answer`.
  """
  @type sdp_type_t() :: :offer | :answer

  @typedoc """
  A UTF-8 encoded string encapsulating either an offer or an answer.
  """
  @type sdp_t() :: String.t()

  @typedoc """
  A UTF-8 encoded string encapsulating an Offer or an Answer in JSON. The keys are as
  follows:

  | key    | type |
  | ------ | ---- |
  | `type` | `offer`, `answer` |
  | `sdp`  | `sdp_t() |
  """
  @type session_description_t() :: String.t()

  @typedoc """
  Possible states of ICE connection.
  """
  @type ice_connection_state_t() ::
          :unspecified
          | :new
          | :checking
          | :connected
          | :completed
          | :disconnected
          | :failed
          | :closed

  @typedoc """
  Message sent as a result of a call to `ice_connection_state/2`.
  """
  @type ice_connection_state_msg_t() ::
          {:ice_connection_state, peer_connection_t(), ice_connection_state_t()}

  @typedoc """
  Possible states of ICE gathering process.
  """
  @type ice_gathering_state_t() :: :complete | :gathering | :new | :unspecified

  @typedoc """
  Message sent as a result of a call to `ice_gathering_state/2`.
  """
  @type ice_gathering_state_msg_t() ::
          {:ice_gathering_state, peer_connection_t(), ice_connection_state_t()}

  @typedoc """
  Possible states of session parameters negotiation.
  """
  @type signaling_state_t() ::
          :closed
          | :have_local_offer
          | :have_local_pranswer
          | :have_remote_offer
          | :have_remote_pranswer
          | :stable
          | :unspecified

  @typedoc """
  Message sent as a result of a call to `signaling_state/2`.
  """
  @type signaling_state_msg_t() :: {:signaling_state, peer_connection_t(), signaling_state_t()}

  @typedoc """
  Possible states of peer connection.
  """
  @type connection_state_t() ::
          :closed | :connected | :connecting | :disconnected | :failed | :new | :unspecified

  @typedoc """
  Message sent as a result of a call to `connection_state/2`.
  """
  @type connection_state_msg_t() :: {:connection_state, peer_connection_t(), connection_state_t()}

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
  def init(args \\ []) do
    with {:ok, native} <- Native.init(args) do
      {:ok, %Specter{native: native}}
    end
  end

  @doc """
  Returns the current configuration for the initialized NIF.

  ## Usage

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.example.com:3478"])
      iex> Specter.config(specter)
      {:ok, %Specter.Config{ice_servers: ["stun:stun.example.com:3478"]}}

  """
  @spec config(t()) :: {:ok, Specter.Config.t()} | {:error, term()}
  def config(%Specter{native: ref}), do: Native.config(ref)

  @doc """
  Given an ICE candidate, add it to the given peer connection. Assumes trickle ICE.
  Candidates must be JSON, with the keys `candidate`, `sdp_mid`, `sdp_mline_index`, and
  `username_fragment`.
  """
  @spec add_ice_candidate(t(), peer_connection_t(), ice_candidate_t()) :: :ok | {:error, term()}
  def add_ice_candidate(%Specter{native: ref}, pc, candidate),
    do: Native.add_ice_candidate(ref, pc, candidate)

  @doc """
  Closes an open instance of an RTCPeerConnection.

  ## Usage

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc}
      ...>
      iex> Specter.close_peer_connection(specter, pc)
      :ok
      iex> {:ok, _pc} =
      ...>     receive do
      ...>       {:peer_connection_closed, ^pc} -> {:ok, pc}
      ...>     after
      ...>       500 -> {:error, :timeout}
      ...>     end
      ...>
      iex> Specter.peer_connection_exists?(specter, pc)
      false
  """
  @spec close_peer_connection(t(), peer_connection_t()) :: :ok | {:error, term()}
  def close_peer_connection(%Specter{native: ref}, pc), do: Native.close_peer_connection(ref, pc)

  @doc """
  Sends back state of peer connection.
  This will send message `t:connection_state_msg_t/0`.
  """
  @spec connection_state(t(), peer_connection_t()) :: :ok | {:error, term()}
  def connection_state(%Specter{native: ref}, pc) do
    Native.connection_state(ref, pc)
  end

  @doc """
  Given an RTCPeerConnection where the remote description has been assigned via
  `set_remote_description/4`, create an answer that can be passed to another connection.

  | param             | type                 | default |
  | ----------------- | -------------------- | ------- |
  | `specter`         | `t()`                | |
  | `peer_connection` | `opaque`             | |
  | `options`         | `answer_options_t()` | voice_activity_detection: false |

  """
  @spec create_answer(t(), peer_connection_t(), answer_options_t()) :: :ok | {:error, term()}
  def create_answer(%Specter{native: ref}, pc, opts \\ []),
    do:
      Native.create_answer(
        ref,
        pc,
        Keyword.get(opts, :voice_activity_detection, false)
      )

  @doc """
  Creates a data channel on an RTCPeerConnection.

  Note: this can be useful when attempting to generate a valid offer, but where no media
  tracks are expected to be sent or received. Callbacks from data channels have not yet
  been implemented.
  """
  @spec create_data_channel(t(), peer_connection_t(), String.t()) :: :ok | {:error, term()}
  def create_data_channel(%Specter{native: ref}, pc, label),
    do: Native.create_data_channel(ref, pc, label)

  @doc """
  Given an RTCPeerConnection, create an offer that can be passed to another connection.

  | param             | type                | default |
  | ----------------- | ------------------- | ------- |
  | `specter`         | `t()`               | |
  | `peer_connection` | `opaque`            | |
  | `options`         | `offer_options_t()` | voice_activity_detection: false |
  |                   |                     | ice_restart: false |

  """
  @spec create_offer(t(), peer_connection_t(), offer_options_t()) :: :ok | {:error, term()}
  def create_offer(%Specter{native: ref}, pc, opts \\ []),
    do:
      Native.create_offer(
        ref,
        pc,
        Keyword.get(opts, :voice_activity_detection, false),
        Keyword.get(opts, :ice_restart, false)
      )

  @doc """
  Sends back the value of the current session description on a peer connection. This will
  send back JSON representing an offer or an answer when the peer connection has had
  `set_local_description/3` called and has successfully negotiated ICE. In all other cases,
  `nil` will be sent.

  See `pending_local_description/2` and `local_description/2`.

  ## Usage

      iex> {:ok, specter} = Specter.init()
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc}
      iex> Specter.current_local_description(specter, pc)
      :ok
      iex> assert_receive {:current_local_description, ^pc, nil}
  """
  @spec current_local_description(t(), peer_connection_t()) :: :ok | {:error, term()}
  def current_local_description(%Specter{native: ref}, pc),
    do: Native.current_local_description(ref, pc)

  @doc """
  Sends back the value of the current remote session description on a peer connection. This will
  send back JSON representing an offer or an answer when the peer connection has had
  `set_remote_description/3` called and has successfully negotiated ICE. In all other cases,
  `nil` will be sent.

  See `current_remote_description/2` and `remote_description/2`.

  ## Usage

      iex> {:ok, specter} = Specter.init()
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc}
      iex> Specter.current_remote_description(specter, pc)
      :ok
      iex> assert_receive {:current_remote_description, ^pc, nil}
  """
  @spec current_remote_description(t(), peer_connection_t()) :: :ok | {:error, term()}
  def current_remote_description(%Specter{native: ref}, pc),
    do: Native.current_remote_description(ref, pc)

  @doc """
  Sends back state of ICE connection for given peer connection.
  This will send message `t:ice_connection_state_msg_t/0`
  """
  @spec ice_connection_state(t(), peer_connection_t()) :: :ok | {:error, term()}
  def ice_connection_state(%Specter{native: ref}, pc) do
    Native.ice_connection_state(ref, pc)
  end

  @doc """
  Sends back state of ICE gathering process.
  This will send message `t:ice_gathering_state_t/0`.
  """
  @spec ice_gathering_state(t(), peer_connection_t()) :: :ok | {:error, term()}
  def ice_gathering_state(%Specter{native: ref}, pc) do
    Native.ice_gathering_state(ref, pc)
  end

  @doc """
  Sends back the value of the local session description on a peer connection. This will
  send back JSON representing an offer or an answer when the peer connection has had
  `set_local_description/3` called. If ICE has been successfully negotated, the current
  local description will be sent back, otherwise the caller will receive the pending
  local description.

  See `current_local_description/2` and `pending_local_description/2`.

  ## Usage

      iex> {:ok, specter} = Specter.init()
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc}
      ...>
      iex> Specter.local_description(specter, pc)
      :ok
      iex> assert_receive {:local_description, ^pc, nil}
      ...>
      iex> :ok = Specter.create_offer(specter, pc)
      iex> assert_receive {:offer, ^pc, offer}
      iex> :ok = Specter.set_local_description(specter, pc, offer)
      iex> assert_receive {:ok, ^pc, :set_local_description}
      ...>
      iex> Specter.local_description(specter, pc)
      :ok
      iex> assert_receive {:local_description, ^pc, ^offer}
  """
  @spec local_description(t(), peer_connection_t()) :: :ok | {:error, term()}
  def local_description(%Specter{native: ref}, pc),
    do: Native.local_description(ref, pc)

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
  def media_engine_exists?(%Specter{native: ref}, media_engine) do
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
  @spec new_api(t(), media_engine_t(), registry_t()) :: {:ok, api_t()} | {:error, term()}
  def new_api(%Specter{native: ref}, media_engine, registry),
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
  @spec new_media_engine(t()) :: {:ok, media_engine_t()} | {:error, term()}
  def new_media_engine(%Specter{native: ref}), do: Native.new_media_engine(ref)

  @doc """
  Creates a new RTCPeerConnection, using an API reference created with `new_api/3`. The
  functionality wrapped by this function is async, so `:ok` is returned immediately.
  Callers should listen for the `{:peer_connection_ready, peer_connection_t()}` message
  to receive the results of this function.

  | param     | type     | default |
  | --------- | -------- | ------- |
  | `specter` | `t()`    | |
  | `api`     | `opaque` | |

  ## Usage

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc} = Specter.new_peer_connection(specter, api)
      ...>
      iex> {:ok, _pc} =
      ...>     receive do
      ...>       {:peer_connection_ready, ^pc} -> {:ok, pc}
      ...>     after
      ...>       500 -> {:error, :timeout}
      ...>     end
  """
  @spec new_peer_connection(t(), api_t()) :: {:ok, peer_connection_t()} | {:error, term()}
  def new_peer_connection(%Specter{native: ref}, api), do: Native.new_peer_connection(ref, api)

  @doc """
  Creates an intercepter registry. This is a user configurable RTP/RTCP pipeline,
  and provides features such as NACKs and RTCP Reports. A registry must be created for
  each peer connection.

  The registry may be combined with a MediaEngine in an API (consuming both). The API
  instance is then used to create RTCPeerConnections.

  Note that creating a registry does **not** take ownership of the media engine.

  ## Usage

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, _registry} = Specter.new_registry(specter, media_engine)
      ...>
      iex> Specter.media_engine_exists?(specter, media_engine)
      true

  """
  @spec new_registry(t(), media_engine_t()) :: {:ok, registry_t()} | {:error, term()}
  def new_registry(%Specter{native: ref}, media_engine),
    do: Native.new_registry(ref, media_engine)

  @doc """
  Returns true or false, depending on whether the RTCPeerConnection is initialized.

  ## Usage

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc}
      iex> Specter.peer_connection_exists?(specter, pc)
      true

      iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
      iex> Specter.peer_connection_exists?(specter, UUID.uuid4())
      false
  """
  @spec peer_connection_exists?(t(), peer_connection_t()) :: boolean() | no_return()
  def peer_connection_exists?(%Specter{native: ref}, peer_connection) do
    case Native.peer_connection_exists(ref, peer_connection) do
      {:ok, value} ->
        value

      {:error, error} ->
        raise "Unable to determine whether peer connection exists:\n#{inspect(error)}"
    end
  end

  @doc """
  Sends back the value of the session description on a peer connection that is pending
  connection, or nil.

  ## Usage

      iex> {:ok, specter} = Specter.init()
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc}
      ...>
      iex> Specter.pending_local_description(specter, pc)
      :ok
      iex> assert_receive {:pending_local_description, ^pc, nil}
      ...>
      iex> :ok = Specter.create_offer(specter, pc)
      iex> assert_receive {:offer, ^pc, offer}
      iex> :ok = Specter.set_local_description(specter, pc, offer)
      iex> assert_receive {:ok, ^pc, :set_local_description}
      ...>
      iex> Specter.pending_local_description(specter, pc)
      :ok
      iex> assert_receive {:pending_local_description, ^pc, ^offer}
  """
  @spec pending_local_description(t(), peer_connection_t()) :: :ok | {:error, term()}
  def pending_local_description(%Specter{native: ref}, pc),
    do: Native.pending_local_description(ref, pc)

  @doc """
  Sends back the value of the remote session description on a peer connection on a peer
  that is pending connection, or nil.

  See `current_remote_description/2` and `pending_remote_description/2`.

  ## Usage

      iex> {:ok, specter} = Specter.init()
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc_offer} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc_offer}
      iex> :ok = Specter.create_data_channel(specter, pc_offer, "foo")
      iex> :ok = Specter.create_offer(specter, pc_offer)
      iex> assert_receive {:offer, ^pc_offer, offer}
      ...>
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc_answer} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc_answer}
      ...>
      iex> Specter.pending_remote_description(specter, pc_answer)
      :ok
      iex> assert_receive {:pending_remote_description, ^pc_answer, nil}
      ...>
      iex> :ok = Specter.set_remote_description(specter, pc_answer, offer)
      iex> assert_receive {:ok, ^pc_answer, :set_remote_description}
      ...>
      iex> Specter.pending_remote_description(specter, pc_answer)
      :ok
      iex> assert_receive {:pending_remote_description, ^pc_answer, ^offer}
  """
  @spec pending_remote_description(t(), peer_connection_t()) :: :ok | {:error, term()}
  def pending_remote_description(%Specter{native: ref}, pc),
    do: Native.pending_remote_description(ref, pc)

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
  def registry_exists?(%Specter{native: ref}, registry) do
    case Native.registry_exists(ref, registry) do
      {:ok, value} ->
        value

      {:error, error} ->
        raise "Unable to determine whether registry exists:\n#{inspect(error)}"
    end
  end

  @doc """
  Sends back the value of the remote session description on a peer connection. This will
  send back JSON representing an offer or an answer when the peer connection has had
  `set_remote_description/3` called. If ICE has been successfully negotated, the current
  remote description will be sent back, otherwise the caller will receive the pending
  remote description.

  See `current_remote_description/2` and `remote_description/2`.

  ## Usage

      iex> {:ok, specter} = Specter.init()
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc_offer} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc_offer}
      iex> :ok = Specter.create_data_channel(specter, pc_offer, "foo")
      iex> assert_receive {:data_channel_created, ^pc_offer}
      iex> :ok = Specter.create_offer(specter, pc_offer)
      iex> assert_receive {:offer, ^pc_offer, offer}
      ...>
      iex> {:ok, media_engine} = Specter.new_media_engine(specter)
      iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
      iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
      iex> {:ok, pc_answer} = Specter.new_peer_connection(specter, api)
      iex> assert_receive {:peer_connection_ready, ^pc_answer}
      ...>
      iex> Specter.remote_description(specter, pc_answer)
      :ok
      iex> assert_receive {:remote_description, ^pc_answer, nil}
      ...>
      iex> :ok = Specter.set_remote_description(specter, pc_answer, offer)
      iex> assert_receive {:ok, ^pc_answer, :set_remote_description}
      ...>
      iex> Specter.remote_description(specter, pc_answer)
      :ok
      iex> assert_receive {:remote_description, ^pc_answer, ^offer}
  """
  @spec remote_description(t(), peer_connection_t()) :: :ok | {:error, term()}
  def remote_description(%Specter{native: ref}, pc),
    do: Native.remote_description(ref, pc)

  @doc """
  Given an offer or an answer session description, sets the local description on
  a peer connection. The description should be in the form of JSON with the keys
  `type` and `sdp`.

  | param             | type                        | default |
  | ----------------- | --------------------------- | ------- |
  | `specter`         | `t:t/0`                     | |
  | `peer_connection` | `opaque`                    | |
  | `description`     | `t:session_description_t()` | |
  """
  @spec set_local_description(t(), peer_connection_t(), session_description_t()) ::
          :ok | {:error, term()}
  def set_local_description(%Specter{native: ref}, pc, description),
    do: Native.set_local_description(ref, pc, description)

  @doc """
  Given an offer or an answer in the form of SDP generated by a remote party, sets
  the remote description on a peer connection. Expects a session description in the
  form of JSON with the keys `type` and `sdp`.

  | param             | type                        | default |
  | ----------------- | --------------------------- | ------- |
  | `specter`         | `t:t/0`                     | |
  | `peer_connection` | `opaque`                    | |
  | `description`     | `t:session_description_t/0` | |
  """
  @spec set_remote_description(t(), peer_connection_t(), session_description_t()) ::
          :ok | {:error, term()}
  def set_remote_description(%Specter{native: ref}, pc, description) do
    Native.set_remote_description(ref, pc, description)
  end

  @doc """
  Sends back state of session parameters negotiation.
  This will send message `t:signaling_state_msg_t/0`.
  """
  @spec signaling_state(t(), peer_connection_t()) :: :ok | {:error, term()}
  def signaling_state(%Specter{native: ref}, pc) do
    Native.signaling_state(ref, pc)
  end
end
