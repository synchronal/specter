defmodule SpecterTest.NifHelpers do
  @moduledoc false
  import ExUnit.Assertions

  def create_data_channel(%{specter: specter, peer_connection: pc}) do
    :ok = Specter.PeerConnection.create_data_channel(specter, pc, "data")
    ExUnit.Assertions.assert_receive({:data_channel_created, ^pc})
    :ok
  end

  def create_offer(%{specter: specter, peer_connection: pc}) do
    :ok = Specter.PeerConnection.create_offer(specter, pc)
    ExUnit.Assertions.assert_receive({:offer, ^pc, offer})
    [offer: offer]
  end

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
    {:ok, pc} = Specter.PeerConnection.new(specter, api)
    assert_receive {:peer_connection_ready, ^pc}
    true = Specter.PeerConnection.exists?(specter, pc)
    pc
  end
end
