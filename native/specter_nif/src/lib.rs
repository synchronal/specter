use rustler::{Env, Term};

mod atoms;
mod state;

fn on_load(env: Env, _info: Term) -> bool {
    rustler::resource!(state::StateResource, env);
    true
}

rustler::init!("Elixir.Specter.Native", [state::init], load = on_load);
