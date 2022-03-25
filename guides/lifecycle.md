Lifecycle
========

* Initialize the library. Register messages to the current pid.
  ```elixir
  {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
  ```

* Create a peer connection's dependencies. Creating an API consumes its MediaEngine
  and Registry, making them unavailable for future API instances.
  ```elixir
  {:ok, media_engine} = Specter.new_media_engine(specter)
  {:ok, registry} = Specter.new_registry(specter, media_engine)

  true = Specter.media_engine_exists?(specter, media_engine)
  true = Specter.registry_exists?(specter, registry)

  {:ok, api} = Specter.new_api(specter, media_engine, registry)

  false = Specter.media_engine_exists?(specter, media_engine)
  false = Specter.registry_exists?(specter, registry)
  ```

* Create a peer connection
  ```elixir
  {:ok, pc_1} = Specter.new_peer_connection(specter, api)
  :ok = receive do: ({:peer_connection_ready, ^pc_1} -> :ok),
      after: (100 -> {:error, :timeout})

  true = Specter.peer_connection_exists?(specter, pc_1)
  ```

* Add a thing to be negotiated
  ```elixir
  :ok = Specter.create_data_channel(specter, pc_1, "data")
  :ok = receive do: ({:data_channel_created, ^pc_1} -> :ok),
      after: (100 -> {:error, :timeout})
  ```

* Create an offer
  ```elixir
  :ok = Specter.create_offer(specter, pc_1)
  {:ok, offer} = receive do: ({:offer, ^pc_1, offer} -> {:ok, offer}),
      after: (100 -> {:error, :timeout})

  :ok = Specter.set_local_description(specter, pc_1, offer)
  :ok = receive do: ({:ok, ^pc_1, :set_local_description} -> :ok),
      after: (100 -> {:error, :timeout})
  ```

* Create a second peer connection, to answer back
  ```elixir
  {:ok, media_engine} = Specter.new_media_engine(specter)
  {:ok, registry} = Specter.new_registry(specter, media_engine)
  {:ok, api} = Specter.new_api(specter, media_engine, registry)
  {:ok, pc_2} = Specter.new_peer_connection(specter, api)
  :ok = receive do: ({:peer_connection_ready, ^pc_2} -> :ok),
      after: (100 -> {:error, :timeout})
  ```

* Begin negotiating offer/answer
  ```elixir
  ##  give the offer to the second peer connection
  :ok = Specter.set_remote_description(specter, pc_2, offer)
  :ok = receive do: ({:ok, ^pc_2, :set_remote_description} -> :ok),
      after: (100 -> {:error, :timeout})

  ##  create an answer
  :ok = Specter.create_answer(specter, pc_2)
  {:ok, answer} = receive do: ({:answer, ^pc_2, answer} -> {:ok, answer}),
      after: (100 -> {:error, :timeout})

  :ok = Specter.set_local_description(specter, pc_2, answer)
  :ok = receive do: ({:ok, ^pc_2, :set_local_description} -> :ok),
      after: (100 -> {:error, :timeout})

  ##  give the answer to the first peer connection
  :ok = Specter.set_remote_description(specter, pc_1, answer)
  :ok = receive do: ({:ok, ^pc_1, :set_remote_description} -> :ok),
      after: (100 -> {:error, :timeout})
  ```

* Receive ice candidates
  ```elixir
  {:ok, candidate} = receive do: ({:ice_candidate, ^pc_1, c} -> {:ok, c}),
      after: (100 -> {:error, :timeout})
  :ok = Specter.add_ice_candidate(specter, pc_2, candidate)

  ## .... and so on.

  {:ok, candidate} = receive do: ({:ice_candidate, ^pc_2, c} -> {:ok, c}),
      after: (100 -> {:error, :timeout})
  :ok = Specter.add_ice_candidate(specter, pc_1, candidate)

  ## .... and so on.
  ```

* Shut everything down
  ```elixir
  iex> Specter.close_peer_connection(specter, pc_1)
  :ok
  iex> assert_receive {:peer_connection_closed, ^pc_1}
  ...>
  iex> :ok = Specter.close_peer_connection(specter, pc_2)
  iex> assert_receive {:peer_connection_closed, ^pc_2}
  ```

