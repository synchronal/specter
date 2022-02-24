defmodule Specter.Config do
  @moduledoc """
  A representation of configuration kept in the initialized NIF.
  """

  defstruct [
    :ice_servers
  ]

  @typedoc """
  A representation of the configuration kept by the initialized NIF.
  Note that this does not map 1:1 to actual webrtc.rs data structures, but
  is an Elixir data structure into which the NIF can encode its config.
  """
  @type t() :: %__MODULE__{
          ice_servers: [Specter.ice_server()]
        }
end
