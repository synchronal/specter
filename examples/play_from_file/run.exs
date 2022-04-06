# helper module until we add support
# for subscribing for various state changes
defmodule Stopwatch do
  def wait_for_ice_gathering_complete(specter, pc) do
    :ok = Specter.PeerConnection.ice_gathering_state(specter, pc)

    receive do
      {:ice_gathering_state, ^pc, :complete} -> IO.puts("ICE gathering complete")
      {:ice_gathering_state, ^pc, _other} -> wait_for_ice_gathering_complete(specter, pc)
    end
  end

  def wait_for_peer_connection_connected(specter, pc) do
    :ok = Specter.PeerConnection.connection_state(specter, pc)

    receive do
      {:connection_state, ^pc, :connected} ->
        IO.puts("Peer connection connected")

      {:connection_state, ^pc, :failed} ->
        IO.puts("Peer connection failed")
        exit(:normal)

      {:connection_state, ^pc, _other} ->
        wait_for_peer_connection_connected(specter, pc)
    end
  end

  def wait_for_peer_connection_ready(pc) do
    receive do
      {:peer_connection_ready, ^pc} -> :ok
      after: 100 -> {:error, :timeout}
    end
  end

  def wait_for_rtp_sender(pc, track) do
    receive do
      {:rtp_sender, ^pc, ^track, rtp_sender} -> {:ok, rtp_sender}
      after: 100 -> {:error, :timeout}
    end
  end

  def wait_for_set_remote_description(pc) do
    receive do
      {:ok, ^pc, :set_remote_description} -> :ok
      after: 100 -> {:error, :timeout}
    end
  end

  def wait_for_answer(pc) do
    receive do
      {:answer, ^pc, answer} -> {:ok, answer}
      after: 100 -> {:error, :timeout}
    end
  end

  def wait_for_set_local_description(pc) do
    receive do
      {:ok, ^pc, :set_local_description} -> :ok
      after: 100 -> {:error, :timeout}
    end
  end

  def wait_for_local_description(pc) do
    receive do
      {:local_description, ^pc, answer} -> {:ok, answer}
      after: 100 -> {:error, :timeout}
    end
  end

  def wait_for_peer_connection_closed(pc) do
    receive do
      {:peer_connection_closed, ^pc} -> :ok
      after: 100 -> {:error, :timeout}
    end
  end

  def wait_for_playback_finished(track) do
    receive do
      {:playback_finished, ^track} -> :ok
    end
  end
end

offer =
  case System.argv() do
    [offer] ->
      Base.decode64!(offer)

    _other ->
      IO.puts("""
      Invalid arguments.

      Usage:
      mix run --no-halt examples/play_from_file.exs <base64_browser_offer_string>
      """)

      exit(:normal)
  end

{:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
{:ok, media_engine} = Specter.new_media_engine(specter)
{:ok, registry} = Specter.new_registry(specter, media_engine)

true = Specter.media_engine_exists?(specter, media_engine)
true = Specter.registry_exists?(specter, registry)

{:ok, api} = Specter.new_api(specter, media_engine, registry)

false = Specter.media_engine_exists?(specter, media_engine)
false = Specter.registry_exists?(specter, registry)

{:ok, pc} = Specter.PeerConnection.new(specter, api)

:ok = Stopwatch.wait_for_peer_connection_ready(pc)

true = Specter.PeerConnection.exists?(specter, pc)

codec = %Specter.RtpCodecCapability{mime_type: "video/H264"}
{:ok, track} = Specter.TrackLocalStaticSample.new(specter, codec, "video", "specter")
:ok = Specter.PeerConnection.add_track(specter, pc, track)
{:ok, _rtp_sender} = Stopwatch.wait_for_rtp_sender(pc, track)

:ok = Specter.PeerConnection.set_remote_description(specter, pc, offer)

:ok = Stopwatch.wait_for_set_remote_description(pc)

:ok = Specter.PeerConnection.create_answer(specter, pc)

{:ok, answer} = Stopwatch.wait_for_answer(pc)

:ok = Specter.PeerConnection.set_local_description(specter, pc, answer)

:ok = Stopwatch.wait_for_set_local_description(pc)

# we don't have to fetch ICE candidates
# they will be included automatically in a local description
Stopwatch.wait_for_ice_gathering_complete(specter, pc)
:ok = Specter.PeerConnection.local_description(specter, pc)

{:ok, answer} = Stopwatch.wait_for_local_description(pc)
IO.puts(Base.encode64(answer))

Stopwatch.wait_for_peer_connection_connected(specter, pc)
:ok = Specter.TrackLocalStaticSample.play_from_file(specter, track, "examples/play_from_file/sample_video.h264")

:ok = Stopwatch.wait_for_playback_finished(track)

:ok = Specter.PeerConnection.close(specter, pc)

:ok = Stopwatch.wait_for_peer_connection_closed(pc)
