defmodule SpecterTest do
  use ExUnit.Case
  doctest Specter

  test "greets the world" do
    assert Specter.hello() == :world
  end
end
