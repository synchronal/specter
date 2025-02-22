use rustler::{Env, Term};

mod atoms;
mod codec_capability;
mod config;
mod peer_connection;
mod state;
mod task;
mod track;
mod util;

fn on_load(_env: Env, _info: Term) -> bool {
    match env_logger::try_init() {
        Ok(()) => log::debug!("Logger initialized succsessfully\r"),
        Err(_reason) => log::debug!("Logger already initialized. Ignoring.\r"),
    };
    true
}

rustler::init!("Elixir.Specter.Native", load = on_load);
