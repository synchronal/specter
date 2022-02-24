defmodule Specter.Config do
  @moduledoc """
  A definition of configuration that is kept in the NIF. Note that
  this does not map 1:1 to actual webrtc.rs data structures, but
  is an Elixir data structure into which the NIF can encode its config.
  """

  defstruct [
    :ice_servers
  ]

  @type t() :: %__MODULE__{
          ice_servers: [Specter.ice_server()]
        }
end
