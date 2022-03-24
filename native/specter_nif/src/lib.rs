use rustler::{Env, Term};

mod atoms;
mod config;
mod peer_connection;
mod state;
mod task;
mod util;

fn on_load(env: Env, _info: Term) -> bool {
    state::load(env);
    true
}

rustler::init!(
    "Elixir.Specter.Native",
    [
        peer_connection::close,
        peer_connection::create_answer,
        peer_connection::create_data_channel,
        peer_connection::create_offer,
        peer_connection::get_current_local_description,
        peer_connection::get_local_description,
        peer_connection::get_pending_local_description,
        peer_connection::new,
        peer_connection::set_local_description,
        peer_connection::set_remote_description,
        state::get_config,
        state::init,
        state::media_engine_exists,
        state::new_api,
        state::new_media_engine,
        state::new_registry,
        state::peer_connection_exists,
        state::registry_exists,
    ],
    load = on_load
);
