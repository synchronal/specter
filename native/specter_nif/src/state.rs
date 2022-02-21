use crate::atoms;
use rustler::{Atom, ResourceArc};
use std::sync::Mutex;

pub struct State {}

impl State {}

pub struct StateResource(Mutex<State>);

#[rustler::nif]
fn init() -> (Atom, ResourceArc<StateResource>) {
    let state = State {};
    let resource = ResourceArc::new(StateResource(Mutex::new(state)));

    (atoms::ok(), resource)
}
