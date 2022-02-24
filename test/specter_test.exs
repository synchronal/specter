defmodule SpecterTest do
  use SpecterTest.Case
  doctest Specter

  @uuid_regex ~r/\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b/i

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

  describe "config" do
    test "returns the current configuration" do
      assert {:ok, ref} =
               Specter.init(
                 ice_servers: [
                   "stun:stun.example.com:3478",
                   "stun:stun.l.example.com:3478"
                 ]
               )

      assert {:ok,
              %Specter.Config{
                ice_servers: [
                  "stun:stun.example.com:3478",
                  "stun:stun.l.example.com:3478"
                ]
              }} = Specter.config(ref)
    end
  end

  describe "new_media_engine" do
    setup :initialize_specter

    test "returns a UUID", %{specter: specter} do
      assert {:ok, uuid} = Specter.new_media_engine(specter)
      assert is_binary(uuid)
      assert String.match?(uuid, @uuid_regex)
    end
  end
end
