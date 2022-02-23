defmodule SpecterTest do
  use ExUnit.Case
  doctest Specter

  describe "init" do
    test "initializes with default configuration" do
      assert {:ok, ref} = Specter.init()
      assert is_reference(ref)
    end

    test "initializes with ice_server configuration" do
      assert {:ok, ref} = Specter.init(ice_servers: ["stun:stun.example.com:3478"])
      assert is_reference(ref)
    end
  end
end
