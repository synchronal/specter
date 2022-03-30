defmodule Test.Integration.IceNegotiationTest do
  use SpecterTest.Case

  describe "on_ice_candidate" do
    setup [:initialize_specter, :init_api, :init_peer_connection]

    test "sends candidates as they are generated", %{specter: specter, peer_connection: pc_offer} do
      api = init_api(specter)
      pc_answer = init_peer_connection(specter, api)

      assert :ok = Specter.PeerConnection.create_data_channel(specter, pc_offer, "foo")
      assert_receive {:data_channel_created, ^pc_offer}
      assert :ok = Specter.PeerConnection.create_offer(specter, pc_offer)
      assert_receive {:offer, ^pc_offer, offer}
      assert :ok = Specter.PeerConnection.set_local_description(specter, pc_offer, offer)
      assert_receive {:ok, ^pc_offer, :set_local_description}

      assert :ok = Specter.PeerConnection.set_remote_description(specter, pc_answer, offer)
      assert_receive {:ok, ^pc_answer, :set_remote_description}

      assert :ok = Specter.PeerConnection.create_answer(specter, pc_answer)
      assert_receive {:answer, ^pc_answer, answer}
      assert :ok = Specter.PeerConnection.set_local_description(specter, pc_answer, answer)
      assert_receive {:ok, ^pc_answer, :set_local_description}

      assert_receive {:ice_candidate, ^pc_offer, _candidate}
      assert_receive {:ice_candidate, ^pc_answer, _candidate}
    end
  end
end
