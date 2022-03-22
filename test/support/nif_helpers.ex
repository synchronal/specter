defmodule SpecterTest.NifHelpers do
  @moduledoc false

  @doc """
  Initialize Specter. Adds `%{specter: specter}` to the test
  context.
  """
  def initialize_specter(_ctx) do
    {:ok, specter} = Specter.init()
    [specter: specter]
  end

  def init_api(%{specter: specter}) do
    [api: init_api(specter)]
  end

  def init_api(%Specter{} = specter) do
    {:ok, media_engine} = Specter.new_media_engine(specter)
    {:ok, registry} = Specter.new_registry(specter, media_engine)
    {:ok, api} = Specter.new_api(specter, media_engine, registry)

    api
  end

  def init_peer_connection(%{specter: specter, api: api}) do
    [peer_connection: init_peer_connection(specter, api)]
  end

  def init_peer_connection(%Specter{} = specter, api) do
    {:ok, pc} = Specter.new_peer_connection(specter, api)

    :ok =
      receive do
        {:peer_connection_ready, ^pc} -> :ok
      after
        1000 ->
          {:error, :timeout}
      end

    true = Specter.peer_connection_exists?(specter, pc)
    pc
  end
end
