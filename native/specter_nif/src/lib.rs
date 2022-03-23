use rustler::{Env, Term};

mod atoms;
mod config;
mod state;
mod task;

fn on_load(env: Env, _info: Term) -> bool {
    state::load(env);
    true
}

rustler::init!(
    "Elixir.Specter.Native",
    [
        state::close_peer_connection,
        state::create_answer,
        state::create_data_channel,
        state::create_offer,
        state::get_config,
        state::init,
        state::media_engine_exists,
        state::new_api,
        state::new_media_engine,
        state::new_peer_connection,
        state::new_registry,
        state::peer_connection_exists,
        state::registry_exists,
        state::set_local_description,
        state::set_remote_description,
    ],
    load = on_load
);
