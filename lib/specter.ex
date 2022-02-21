defmodule Specter do
  @moduledoc """
  Documentation for `Specter`.
  """

  alias Specter.Native

  @spec init() :: {:ok, reference()}
  def init, do: Native.init()
end
