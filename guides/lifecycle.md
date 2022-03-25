Lifecycle
========

* Initialize the library. Register messages to the current pid.
  * `Specter.init/1`
  ```elixir
iex> {:ok, specter} = Specter.init(ice_servers: ["stun:stun.l.google.com:19302"])
  ```

* Create a peer connection's dependencies. Creating an API consumes its MediaEngine
  and Registry, making them unavailable for future API instances.
  * `Specter.new_media_engine/1`
  * `Specter.new_registry/2`
  * `Specter.new_api/3`
  * `Specter.media_engine_exists?/2`
  * `Specter.registry_exists?/2`
  ```elixir
iex> {:ok, media_engine} = Specter.new_media_engine(specter)
iex> {:ok, registry} = Specter.new_registry(specter, media_engine)

iex> true = Specter.media_engine_exists?(specter, media_engine)
iex> true = Specter.registry_exists?(specter, registry)

iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)

iex> false = Specter.media_engine_exists?(specter, media_engine)
iex> false = Specter.registry_exists?(specter, registry)
  ```

* Create a peer connection
  * `Specter.new_peer_connection/2`
  * `Specter.peer_connection_exists?/2`
  ```elixir
iex> {:ok, pc_1} = Specter.new_peer_connection(specter, api)
iex> :ok = receive do: ({:peer_connection_ready, ^pc_1} -> :ok),
...>    after: (100 -> {:error, :timeout})

iex> true = Specter.peer_connection_exists?(specter, pc_1)
  ```

* Add a thing to be negotiated
  * `Specter.create_data_channel/3`
  ```elixir
iex> :ok = Specter.create_data_channel(specter, pc_1, "data")
iex> :ok = receive do: ({:data_channel_created, ^pc_1} -> :ok),
...>    after: (100 -> {:error, :timeout})
  ```

* Create an offer
  * `Specter.create_offer/2`
  * `Specter.set_local_description/3`
  ```elixir
iex> :ok = Specter.create_offer(specter, pc_1)
iex> {:ok, offer} = receive do: ({:offer, ^pc_1, offer} -> {:ok, offer}),
...>    after: (100 -> {:error, :timeout})

iex> :ok = Specter.set_local_description(specter, pc_1, offer)
iex> :ok = receive do: ({:ok, ^pc_1, :set_local_description} -> :ok),
...>    after: (100 -> {:error, :timeout})
  ```

* Create a second peer connection, to answer back
  ```elixir
iex> {:ok, media_engine} = Specter.new_media_engine(specter)
iex> {:ok, registry} = Specter.new_registry(specter, media_engine)
iex> {:ok, api} = Specter.new_api(specter, media_engine, registry)
iex> {:ok, pc_2} = Specter.new_peer_connection(specter, api)
iex> :ok = receive do: ({:peer_connection_ready, ^pc_2} -> :ok),
...>    after: (100 -> {:error, :timeout})
```

* Begin negotiating offer/answer
  * `Specter.set_remote_description/3`
  * `Specter.create_answer/2`
  * `Specter.set_local_description/3`
  ```elixir
##  give the offer to the second peer connection
iex> :ok = Specter.set_remote_description(specter, pc_2, offer)
iex> :ok = receive do: ({:ok, ^pc_2, :set_remote_description} -> :ok),
...>    after: (100 -> {:error, :timeout})

##  create an answer
iex> :ok = Specter.create_answer(specter, pc_2)
iex> {:ok, answer} = receive do: ({:answer, ^pc_2, answer} -> {:ok, answer}),
...>    after: (100 -> {:error, :timeout})

iex> :ok = Specter.set_local_description(specter, pc_2, answer)
iex> :ok = receive do: ({:ok, ^pc_2, :set_local_description} -> :ok),
...>    after: (100 -> {:error, :timeout})

##  give the answer to the first peer connection
iex> :ok = Specter.set_remote_description(specter, pc_1, answer)
iex> :ok = receive do: ({:ok, ^pc_1, :set_remote_description} -> :ok),
...>    after: (100 -> {:error, :timeout})
```

* Receive ice candidates
  * `Specter.add_ice_candidate/3`
  ```elixir
iex> {:ok, candidate} = receive do: ({:ice_candidate, ^pc_1, c} -> {:ok, c}),
...>    after: (100 -> {:error, :timeout})
iex> :ok = Specter.add_ice_candidate(specter, pc_2, candidate)

## .... and so on.

iex> {:ok, candidate} = receive do: ({:ice_candidate, ^pc_2, c} -> {:ok, c}),
...>    after: (100 -> {:error, :timeout})
iex>:ok = Specter.add_ice_candidate(specter, pc_1, candidate)

## .... and so on.
```

* Shut everything down
  * `Specter.close_peer_connection/2`
  ```elixir
iex> :ok = Specter.close_peer_connection(specter, pc_1)
iex> receive do: ({:peer_connection_closed, ^pc_1} -> :ok),
...>    after: (100 -> {:error, :timeout})
:ok

iex> :ok = Specter.close_peer_connection(specter, pc_2)
iex> receive do: ({:peer_connection_closed, ^pc_2} -> :ok),
...>    after: (100 -> {:error, :timeout})
:ok
```

