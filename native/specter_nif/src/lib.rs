use rustler::{Env, Term};

mod atoms;
mod specter_config;
mod state;

fn on_load(env: Env, _info: Term) -> bool {
    rustler::resource!(state::StateResource, env);
    true
}

rustler::init!(
    "Elixir.Specter.Native",
    [state::init, state::config],
    load = on_load
);
