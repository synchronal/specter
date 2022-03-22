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

  describe "new_peer_connection" do
    setup [:initialize_specter, :init_api]

    test "returns a UUID, then sends a :peer_connection_ready", %{specter: specter, api: api} do
      assert {:ok, pc} = Specter.new_peer_connection(specter, api)
      assert_receive {:peer_connection_ready, ^pc}

      assert is_binary(pc)
      assert String.match?(pc, @uuid_regex)
    end

    test "returns {:error, :not_found} when given a random api id", %{specter: specter} do
      assert {:error, :not_found} = Specter.new_peer_connection(specter, UUID.uuid4())
    end
  end

  describe "close_peer_connection" do
    setup [:initialize_specter, :init_api]

    test "returns {:error, :not_found} when given a random api id", %{specter: specter} do
      assert {:error, :not_found} = Specter.close_peer_connection(specter, UUID.uuid4())
    end

    test "returns :ok, then receives a closed message", %{specter: specter, api: api} do
      assert {:ok, pc} = Specter.new_peer_connection(specter, api)
      assert_receive {:peer_connection_ready, ^pc}

      assert :ok = Specter.close_peer_connection(specter, pc)
      assert_receive {:peer_connection_closed, ^pc}
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

  describe "peer_connection_exists?" do
    setup [:initialize_specter, :init_api]

    test "is false when the peer connection does not exist", %{specter: specter} do
      refute Specter.peer_connection_exists?(specter, UUID.uuid4())
    end

    test "is true when the peer connection exists", %{specter: specter, api: api} do
      assert {:ok, pc} = Specter.new_peer_connection(specter, api)
      refute Specter.peer_connection_exists?(specter, pc)
      assert_receive {:peer_connection_ready, ^pc}

      assert Specter.peer_connection_exists?(specter, pc)
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

  describe "set_remote_description" do
    setup [:initialize_specter, :init_api, :init_peer_connection]

    @valid_offer """
    v=0
    o=- 2927307686215094172 2 IN IP4 127.0.0.1
    s=-
    t=0 0
    a=extmap-allow-mixed
    a=msid-semantic: WMS
    a=ice-ufrag:ZZZZ
    a=ice-pwd:AU/SQPupllyS0SDG/eRWDCfA
    a=fingerprint:sha-256 B7:D5:86:B0:92:C6:A6:03:80:C8:59:47:25:EC:FF:3F:57:F5:97:EF:76:B9:AA:14:B7:8C:C9:B3:4D:CA:1B:0A
    """

    test "returns :ok when given an offer", %{specter: specter, peer_connection: peer_connection} do
      assert :ok = Specter.set_remote_description(specter, peer_connection, :offer, @valid_offer)
      assert_receive {:ok, ^peer_connection, :set_remote_description}
      refute_received {:error, ^peer_connection, :invalid_remote_description}
    end

    test "sends an error message when SDP in invalid", %{
      specter: specter,
      peer_connection: peer_connection
    } do
      offer = "Hello world"
      assert :ok = Specter.set_remote_description(specter, peer_connection, :offer, offer)

      assert_receive {:invalid_remote_description, ^peer_connection,
                      "SdpInvalidSyntax: Hello world"}
    end

    test "returns an error when peer connection does not exist", %{specter: specter} do
      assert {:error, :not_found} =
               Specter.set_remote_description(specter, UUID.uuid4(), :offer, @valid_offer)
    end
  end

  describe "create_offer" do
    setup [:initialize_specter, :init_api, :init_peer_connection]

    test "returns an error when peer connection does not exist", %{specter: specter} do
      assert {:error, :not_found} = Specter.create_offer(specter, UUID.uuid4())
    end

    test "returns :ok and then sends an offer", %{
      specter: specter,
      peer_connection: peer_connection
    } do
      assert :ok = Specter.create_offer(specter, peer_connection)
      assert_receive {:offer, ^peer_connection, offer}
      assert is_binary(offer)
    end

    test "returns :ok with VAD", %{
      specter: specter,
      peer_connection: peer_connection
    } do
      assert :ok = Specter.create_offer(specter, peer_connection, voice_activity_detection: true)
      assert_receive {:offer, ^peer_connection, offer}
      assert is_binary(offer)

      # assert offer is different... somehow? maybe after more interactions are available, the generated
      # SDP will actually be different.
    end

    test "returns error with ice_restart before ICE has started", %{
      specter: specter,
      peer_connection: peer_connection
    } do
      assert :ok = Specter.create_offer(specter, peer_connection, ice_restart: true)
      assert_receive {:offer_error, ^peer_connection, "ICEAgent does not exist"}
    end
  end
end
