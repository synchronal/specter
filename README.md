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
- [ ] `Specter.new_rtc_peer_connection/1` (ref), returning UUID
- [ ] `Specter.set_remote_description/3` (ref, uuid, offer)
- [ ] `Specter.create_answer/3` (ref, uuid, opts)
  - opts: (`voice_activity_detection`: `bool`)
- [ ] `Specter.set_local_description`
- [ ] `pc.on_ice_candidate` sends candidate to callback process
- [ ] `pc.gathering_complete_promise` sends message to callback process
- [ ] `Specter.set_description`
- [ ] `Specter.create_offer/3` (ref, uuid, opts)
  - opts: (`voice_activity_detection`: `bool`, `ice_restart`: `bool`)
- [ ] `Specter.local_description/2`
- [ ] `Specter.close/2`  (ref, uuid)

