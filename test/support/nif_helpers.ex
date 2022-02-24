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
end
