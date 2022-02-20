use rustler::{Atom, ResourceArc};
use std::sync::Mutex;
use crate::atoms;

pub struct State {
    value: String
}

impl State {
    pub fn get(&self) -> String {
        self.value.clone()
    }

    pub fn update(&mut self, value: String) -> &mut State {
        self.value = value.clone();
        self
    }
}

pub struct StateResource(Mutex<State>);


#[rustler::nif]
fn get(resource: ResourceArc<StateResource>) -> Result<String, Atom> {
    let state = match resource.0.try_lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    Ok(state.get())
}

#[rustler::nif]
fn set(resource: ResourceArc<StateResource>, value: String) -> Result<String, Atom> {
    let mut state = match resource.0.try_lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    state.update(value.clone());

    Ok(value)
}

#[rustler::nif]
fn new() -> (Atom, ResourceArc<StateResource>) {
    let state = State { value: "Hello world".to_string() };
    let resource = ResourceArc::new(StateResource(Mutex::new(state)));

    (atoms::ok(), resource)
}

