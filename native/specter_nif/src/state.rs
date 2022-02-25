use crate::atoms;
use crate::specter_config::SpecterConfig;
use rustler::{Atom, Encoder, Env, ResourceArc, Term};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;

pub struct State {
    config: RTCConfiguration,
    media_engines: HashMap<String, MediaEngine>,
    registries: HashMap<String, Registry>,
}

impl State {
    pub(crate) fn add_media_engine(&mut self, uuid: &String, engine: MediaEngine) -> &mut State {
        self.media_engines.insert(uuid.clone(), engine);
        self
    }

    pub(crate) fn get_media_engine(&mut self, uuid: &String) -> Option<&mut MediaEngine> {
        self.media_engines.get_mut(uuid)
    }

    pub(crate) fn add_registry(&mut self, uuid: &String, registry: Registry) -> &mut State {
        self.registries.insert(uuid.clone(), registry);
        self
    }

    pub(crate) fn _get_registry(&mut self, uuid: &String) -> Option<&mut Registry> {
        self.registries.get_mut(uuid)
    }
}

// The resource which will be wrapped in an ResourceArc and returned to
// Elixir as a reference.:w
pub struct StateResource(Mutex<State>);

// Initialize the NIF, returning a reference to Elixir that can be
// passed back into the NIF to retrieve or alter state.
#[rustler::nif(name = "__init__")]
fn init<'a>(env: Env<'a>, opts: Term<'a>) -> Term<'a> {
    let rtc_config = match parse_init_opts(env, opts) {
        Err(error) => return (atoms::error(), error).encode(env),
        Ok(rtc_config) => rtc_config,
    };

    let state = State {
        config: rtc_config,
        media_engines: HashMap::new(),
        registries: HashMap::new(),
    };
    let resource = ResourceArc::new(StateResource(Mutex::new(state)));

    (atoms::ok(), resource).encode(env)
}

#[rustler::nif]
fn config<'a>(env: Env<'a>, resource: ResourceArc<StateResource>) -> Result<Term<'a>, Atom> {
    let state = match resource.0.try_lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    let nif_config = SpecterConfig::from_rtc_configuration(&state.config);
    Ok(nif_config.encode(env))
}

// Create a MediaEngine object to configure the default supported codecs.
//
// Open questions:
// - What actually is a MediaEngine?
// - Why is it here?
// - Do we ever interact with it later, or is it just used to configure
//   behaviors of RTCPeerConnections?
#[rustler::nif]
fn new_media_engine<'a>(
    env: Env<'a>,
    resource: ResourceArc<StateResource>,
) -> Result<Term<'a>, Atom> {
    let mut state = match resource.0.try_lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    let mut m = MediaEngine::default();
    match m.register_default_codecs() {
        Err(_error) => return Err(atoms::webrtc_error()),
        Ok(term) => term,
    }

    let engine_id = Uuid::new_v4().to_hyphenated().to_string();
    state.add_media_engine(&engine_id, m);
    Ok(engine_id.encode(env))
}

// Create an intercepter registry.
//
// Open questions:
// - What the heck is an intercepter registry?
// - How is it used later?
#[rustler::nif]
fn new_registry<'a>(
    env: Env<'a>,
    resource: ResourceArc<StateResource>,
    media_engine_uuid: Term<'a>,
) -> Result<Term<'a>, Atom> {
    let mut state = match resource.0.try_lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    // This could stand some error handling. The match implementation
    // fails with "creates a temporary which is freed while still in use."
    let mid = &media_engine_uuid.clone().decode().unwrap();
    let media_engine = match state.get_media_engine(mid) {
        None => return Err(atoms::not_found()),
        Some(m) => m,
    };

    let mut registry = Registry::new();
    registry = match register_default_interceptors(registry, media_engine) {
        Err(_error) => return Err(atoms::webrtc_error()),
        Ok(term) => term,
    };

    let registry_id = Uuid::new_v4().to_hyphenated().to_string();
    state.add_registry(&registry_id, registry);
    Ok(registry_id.encode(env))
}

// private

fn parse_init_opts<'a>(env: Env<'a>, opts: Term<'a>) -> Result<RTCConfiguration, Atom> {
    if !opts.is_map() {
        return Err(atoms::invalid_configuration());
    };

    let ice_servers = match opts.map_get(atoms::ice_servers().to_term(env)) {
        Err(_) => return Err(atoms::invalid_configuration()),
        Ok(servers) => servers.decode().unwrap(),
    };

    let rtc_config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: ice_servers,
            ..Default::default()
        }],
        ..Default::default()
    };

    Ok(rtc_config)
}
