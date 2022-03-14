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
    {:ok, media_engine} = Specter.new_media_engine(specter)
    {:ok, registry} = Specter.new_registry(specter, media_engine)
    {:ok, api} = Specter.new_api(specter, media_engine, registry)

    [api: api]
  end
end
