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
      assert {:ok, media_engine} = Specter.new_media_engine(specter)
      assert is_binary(media_engine)
      assert String.match?(media_engine, @uuid_regex)
    end
  end

  describe "new_registry" do
    setup :initialize_specter

    test "returns a UUID", %{specter: specter} do
      assert {:ok, media_engine} = Specter.new_media_engine(specter)
      assert {:ok, registry} = Specter.new_registry(specter, media_engine)
      assert is_binary(registry)
      assert String.match?(registry, @uuid_regex)
    end

    test "returns {:error, :not_found} when given a random media engine id", %{specter: specter} do
      assert {:error, :not_found} = Specter.new_registry(specter, UUID.uuid4())
    end
  end
end
