use rustler::{Env, Term};

mod atoms;
mod state;


// LOAD NIF

fn on_load(env: Env, _info: Term) -> bool {
    rustler::resource!(state::StateResource, env);
    true
}

rustler::init!(
    "Elixir.Specter.NIF",
    [state::new, state::get, state::set],
    load=on_load
);
