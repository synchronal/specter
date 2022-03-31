defmodule SpecterTest do
  use SpecterTest.Case
  doctest Specter

  @uuid_regex ~r/\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b/i

  describe "init" do
    test "initializes with default configuration" do
      assert {:ok, %Specter{native: ref}} = Specter.init()
      assert is_reference(ref)
    end

    test "initializes with ice_server configuration" do
      assert {:ok, %Specter{native: ref}} =
               Specter.init(ice_servers: ["stun:stun.example.com:3478"])

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

  describe "new_api" do
    setup :initialize_specter

    test "returns a UUID, and consumes the media engine and registry", %{specter: specter} do
      assert {:ok, media_engine} = Specter.new_media_engine(specter)
      assert {:ok, registry} = Specter.new_registry(specter, media_engine)
      assert {:ok, api_builder} = Specter.new_api(specter, media_engine, registry)
      assert is_binary(api_builder)
      assert String.match?(api_builder, @uuid_regex)

      refute Specter.media_engine_exists?(specter, media_engine)
      refute Specter.registry_exists?(specter, registry)
    end

    test "returns {:error, :not_found} when given a random media engine id", %{specter: specter} do
      assert {:ok, media_engine} = Specter.new_media_engine(specter)
      assert {:ok, registry} = Specter.new_registry(specter, media_engine)

      assert {:error, :not_found} = Specter.new_api(specter, UUID.uuid4(), registry)
    end

    test "returns {:error, :not_found} when given a random registry id", %{specter: specter} do
      assert {:ok, media_engine} = Specter.new_media_engine(specter)
      assert {:ok, _registry} = Specter.new_registry(specter, media_engine)

      assert {:error, :not_found} = Specter.new_api(specter, media_engine, UUID.uuid4())
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

  describe "media_engine_exists?" do
    setup :initialize_specter

    test "is false when the media engine does not exist", %{specter: specter} do
      refute Specter.media_engine_exists?(specter, UUID.uuid4())
    end

    test "is true when the media engine exists", %{specter: specter} do
      assert {:ok, media_engine} = Specter.new_media_engine(specter)
      assert Specter.media_engine_exists?(specter, media_engine)
    end
  end

  describe "registry_exists?" do
    setup :initialize_specter

    test "is false when the registry does not exist", %{specter: specter} do
      refute Specter.registry_exists?(specter, UUID.uuid4())
    end

    test "is true when the media engine exists", %{specter: specter} do
      assert {:ok, media_engine} = Specter.new_media_engine(specter)
      assert {:ok, registry} = Specter.new_registry(specter, media_engine)
      assert Specter.registry_exists?(specter, registry)
    end
  end

  describe "new_track_local_static_sample" do
    setup [:initialize_specter, :init_api, :init_peer_connection]

    test "creates new track properly and returns its uuid", %{specter: specter} do
      codec = %Specter.RtpCodecCapability{mime_type: "audio"}

      assert {:ok, track_uuid} =
               Specter.TrackLocalStaticSample.new(specter, codec, "audio", "specter")

      assert is_binary(track_uuid)
    end

    test "adds new track properly and returns uuid of newly created rtp sender", %{
      specter: specter,
      peer_connection: peer_connection
    } do
      codec = %Specter.RtpCodecCapability{mime_type: "audio"}

      assert {:ok, track_uuid} =
               Specter.TrackLocalStaticSample.new(specter, codec, "audio", "specter")

      assert :ok = Specter.PeerConnection.add_track(specter, peer_connection, track_uuid)
      assert_receive {:rtp_sender, ^peer_connection, ^track_uuid, rtp_sender_uuid}
      assert is_binary(rtp_sender_uuid)
    end
  end
end
