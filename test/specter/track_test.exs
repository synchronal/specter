defmodule Specter.TrackLocalStaticSampleTest do
  use SpecterTest.Case
  doctest Specter.TrackLocalStaticSample

  describe "new_track_local_static_sample" do
    setup [:initialize_specter, :init_api, :init_peer_connection]

    test "creates new track properly and returns its uuid", %{specter: specter} do
      codec = %Specter.RtpCodecCapability{mime_type: "audio"}

      assert {:ok, track_uuid} =
               Specter.TrackLocalStaticSample.new(specter, codec, "audio", "specter")

      assert is_binary(track_uuid)
    end
  end
end
