defmodule SpecterTest.Case do
  @moduledoc """
  An ExUnit test case providing helpers for writing Specter tests.
  """

  use ExUnit.CaseTemplate

  using do
    quote do
      import SpecterTest.NifHelpers
    end
  end
end
