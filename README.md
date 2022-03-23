# Specter

Wrapper for [webrtc.rs](https://webrtc.rs) as an Elixir NIF, using
Rustler.

This library is a low-level interface to the data structures and
entities provided by Rust, with a minimal set of opinions.


## Installation

```elixir
def deps do
  [
    {:specter, "~> 0.1.0"}
  ]
end
```

## Checklist

- [x] `Specter.init/1` takes (opts)
  - opts: (`ice_servers`)
- [x] `Specter.config/1` (ref), returning `Specter.Config.t()`
- [x] `Specter.new_media_engine/1` (ref), returning UUID
- [x] `Specter.new_registry/2` (ref, uuid), returning UUID
- [x] `Specter.new_api/3` (ref, uuid, uuid), returning UUID
  - arg1: media engine uuid
  - arg2: registry uuid
- [x] `Specter.new_peer_connection/2` (ref, uuid), returning UUID
  - arg1: api builder uuid
- [x] `Specter.set_remote_description/3` (ref, uuid, json)
- [x] `Specter.create_offer/3` (ref, uuid, opts)
  - opts: (`voice_activity_detection`: `bool`, `ice_restart`: `bool`)
- [x] `Specter.create_data_channel/3` (ref, uuid, label)
- [x] `Specter.create_answer/3` (ref, uuid, opts)
  - opts: (`voice_activity_detection`: `bool`)
- [x] `Specter.set_local_description/3` (ref, uuid, json)
- [ ] `Specter.local_description`
- [ ] `Specter.current_remote_description`
- [ ] `Specter.pending_remote_description`
- [ ] `Specter.subscribe_to_state_changes` (ref, uuid)
  - state changes sent to Elixir pid
- [ ] `pc.on_ice_candidate` sends candidate to callback process
- [ ] `pc.gathering_complete_promise` sends message to callback process
- [ ] `Specter.set_description`
- [ ] `Specter.close`  (ref, uuid)


## Development

Development of `Specter` depends on Elixir, Erlang, and Rust being available
in the environment. Suggested setup:

```shell
asdf plugin-add erlang
asdf plugin-add elixir
asdf plugin-add rust

bin/dev/doctor
```

CI will run tests and audit the repository, but to make sure all checks locally,
the following commands can be run:

```shell
bin/dev/audit
bin/dev/test
```

The following script is encouraged to run all checks as a part of pushing commits:

```shell
bin/dev/shipit
```


## References / Thank yous

This project is indebted to the following sites and references (at the very least).

- https://github.com/scrogson/franz

