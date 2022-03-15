use crate::atoms;
use crate::config::Config;
use crate::task;
use rustler::types::pid::Pid;
use rustler::{Atom, Encoder, Env, ResourceArc, Term};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use webrtc::api::interceptor_registry as interceptor;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::{APIBuilder, API};
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;

pub struct State {
    apis: HashMap<String, Arc<API>>,
    config: Config,
    media_engines: HashMap<String, MediaEngine>,
    peer_connections: HashMap<String, RTCPeerConnection>,
    pid: Pid,
    registries: HashMap<String, Registry>,
}

impl State {
    fn new(config: Config, pid: Pid) -> Self {
        State {
            config,
            pid,
            apis: HashMap::new(),
            media_engines: HashMap::new(),
            peer_connections: HashMap::new(),
            registries: HashMap::new(),
        }
    }

    //***** API

    pub(crate) fn add_api(&mut self, uuid: &String, api: API) -> &mut State {
        self.apis.insert(uuid.clone(), Arc::new(api));
        self
    }

    pub(crate) fn get_api<'a>(&self, uuid: Term<'a>) -> Option<&Arc<API>> {
        let aid: &String = &uuid.clone().decode().unwrap();
        self.apis.get(aid)
    }

    //***** MediaEngine

    pub(crate) fn add_media_engine(&mut self, uuid: &String, engine: MediaEngine) -> &mut State {
        self.media_engines.insert(uuid.clone(), engine);
        self
    }

    pub(crate) fn get_media_engine<'a>(&mut self, uuid: Term<'a>) -> Option<&MediaEngine> {
        // This could stand some error handling. The match implementation
        // fails with "creates a temporary which is freed while still in use."
        let mid: &String = &uuid.clone().decode().unwrap();
        self.media_engines.get(mid)
    }

    pub(crate) fn get_media_engine_mut<'a>(&mut self, uuid: Term<'a>) -> Option<&mut MediaEngine> {
        let mid: &String = &uuid.clone().decode().unwrap();
        self.media_engines.get_mut(mid)
    }

    pub(crate) fn remove_media_engine<'a>(&mut self, uuid: Term<'a>) -> Option<MediaEngine> {
        let mid: &String = &uuid.clone().decode().unwrap();
        self.media_engines.remove(mid)
    }

    //***** RTCPeerConnection

    pub(crate) fn add_peer_connection(
        &mut self,
        uuid: &String,
        pc: RTCPeerConnection,
    ) -> &mut State {
        self.peer_connections.insert(uuid.clone(), pc);
        self
    }

    pub(crate) fn get_peer_connection<'a>(&self, uuid: Term<'a>) -> Option<&RTCPeerConnection> {
        let aid: &String = &uuid.clone().decode().unwrap();
        self.peer_connections.get(aid)
    }

    //***** Registry

    pub(crate) fn add_registry(&mut self, uuid: &String, registry: Registry) -> &mut State {
        self.registries.insert(uuid.clone(), registry);
        self
    }

    pub(crate) fn get_registry<'a>(&mut self, uuid: Term<'a>) -> Option<&Registry> {
        let rid: &String = &uuid.clone().decode().unwrap();
        self.registries.get(rid)
    }

    pub(crate) fn remove_registry<'a>(&mut self, uuid: Term<'a>) -> Option<Registry> {
        let rid: &String = &uuid.clone().decode().unwrap();
        self.registries.remove(rid)
    }
}

// The resource which will be wrapped in an ResourceArc and returned to
// Elixir as a reference.
pub struct Ref(Arc<Mutex<State>>);

pub fn load(env: Env) -> bool {
    rustler::resource!(Ref, env);
    true
}

/// Initialize the NIF, returning a reference to Elixir that can be
/// passed back into the NIF to retrieve or alter state.
#[rustler::nif(name = "__init__")]
fn init<'a>(env: Env<'a>, opts: Term<'a>) -> Term<'a> {
    let config = match parse_init_opts(env, opts) {
        Err(error) => return (atoms::error(), error).encode(env),
        Ok(config) => config,
    };

    let state = State::new(config, env.pid());
    let resource = ResourceArc::new(Ref(Arc::new(Mutex::new(state))));

    (atoms::ok(), resource).encode(env)
}

#[rustler::nif(name = "config")]
fn get_config<'a>(env: Env<'a>, resource: ResourceArc<Ref>) -> Result<Term<'a>, Atom> {
    let state = match resource.0.try_lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    let config = &state.config;
    Ok(config.encode(env))
}

/// Create a MediaEngine object to configure the default supported codecs.
///
/// Open questions:
/// - What actually is a MediaEngine?
/// - Why is it here?
/// - Do we ever interact with it later, or is it just used to configure
///   behaviors of RTCPeerConnections?
#[rustler::nif]
fn new_media_engine<'a>(env: Env<'a>, resource: ResourceArc<Ref>) -> Result<Term<'a>, Atom> {
    let mut state = match resource.0.try_lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    let mut m = MediaEngine::default();
    match m.register_default_codecs() {
        Err(_error) => return Err(atoms::webrtc_error()),
        Ok(term) => term,
    }

    let engine_id = gen_uuid();
    state.add_media_engine(&engine_id, m);
    Ok(engine_id.encode(env))
}

/// Create an intercepter registry.
///
/// Open questions:
/// - What the heck is an intercepter registry?
/// - How is it used later?
#[rustler::nif]
fn new_registry<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    media_engine_uuid: Term<'a>,
) -> Result<Term<'a>, Atom> {
    let mut state = match resource.0.try_lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    let media_engine = match state.get_media_engine_mut(media_engine_uuid) {
        None => return Err(atoms::not_found()),
        Some(m) => m,
    };

    let mut registry = Registry::new();
    registry = match interceptor::register_default_interceptors(registry, media_engine) {
        Err(_error) => return Err(atoms::webrtc_error()),
        Ok(term) => term,
    };

    let registry_id = gen_uuid();
    state.add_registry(&registry_id, registry);
    Ok(registry_id.encode(env))
}

/// Create a new API. This is directly used when creating RTCPeerConnections.
///
/// Open questions:
/// - This is used to create RTCPeerConnections. Is it used for anything else?
/// - I thought we needed to create a new registry for each PC. Does that
///   mean we need to create a new one of these for each PC?
#[rustler::nif]
fn new_api<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    media_engine_uuid: Term<'a>,
    registry_uuid: Term<'a>,
) -> Result<Term<'a>, Atom> {
    let mut state = match resource.0.try_lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    let media_engine = match state.remove_media_engine(media_engine_uuid) {
        None => return Err(atoms::not_found()),
        Some(m) => m,
    };

    let registry = match state.remove_registry(registry_uuid) {
        None => return Err(atoms::not_found()),
        Some(r) => r,
    };

    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    let api_id = gen_uuid();
    state.add_api(&api_id, api);
    Ok(api_id.encode(env))
}

/// Create a new RTCPeerConnection.
///
/// Open questions:
/// - Once this is initialized, how does it run in a thread that doesn't conflict
///   with the Erlang scheduler?
#[rustler::nif]
fn new_peer_connection<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    api_uuid: Term<'a>,
) -> Term<'a> {
    let mut msg_env = rustler::env::OwnedEnv::new();

    let api = {
        let state_ref = resource.0.lock().unwrap();
        match state_ref.get_api(api_uuid) {
            None => return (atoms::error(), atoms::not_found()).encode(env),
            Some(a) => Arc::clone(a),
        }
    };

    let uuid = gen_uuid();
    let pc_id = uuid.clone();

    task::spawn(async move {
        let rtc_config = {
            let state = resource.0.lock().unwrap();
            RTCConfiguration::from(&state.config.clone())
        };

        let pc = match api.new_peer_connection(rtc_config).await {
            Err(_) => {
                let state = resource.0.lock().unwrap();
                msg_env.send_and_clear(&state.pid, |env| {
                    (atoms::peer_connection_error(), pc_id).encode(env)
                });
                return ();
            }
            Ok(pc) => pc,
        };

        {}

        let mut state = resource.0.lock().unwrap();
        state.add_peer_connection(&pc_id, pc);
        msg_env.send_and_clear(&state.pid, |env| {
            (atoms::peer_connection_ready(), pc_id).encode(env)
        });
    });

    (atoms::ok(), uuid).encode(env)
}

/// Returns true or false depending on whether the State hashmap owns a MediaEngine
/// for the given UUID.
///
/// Some entities need to take ownership of a given MediaEngine, for example when
/// creating an API. After that happens, the MediaEngine will no longer be available
/// in the State hashmap.
#[rustler::nif]
fn media_engine_exists<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    media_engine_uuid: Term<'a>,
) -> Term<'a> {
    let mut state = match resource.0.try_lock() {
        Err(_) => return (atoms::error(), atoms::lock_fail()).encode(env),
        Ok(guard) => guard,
    };

    match state.get_media_engine(media_engine_uuid) {
        None => (atoms::ok(), false).encode(env),
        Some(_m) => (atoms::ok(), true).encode(env),
    }
}
///
/// Returns true or false depending on whether the State hashmap owns an RTCPeerConnection
/// for the given UUID.
#[rustler::nif]
fn peer_connection_exists<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
) -> Term<'a> {
    let state = match resource.0.try_lock() {
        Err(_) => return (atoms::error(), atoms::lock_fail()).encode(env),
        Ok(guard) => guard,
    };

    match state.get_peer_connection(pc_uuid) {
        None => (atoms::ok(), false).encode(env),
        Some(_m) => (atoms::ok(), true).encode(env),
    }
}

/// Returns true or false depending on whether the State hashmap owns a Registry
/// for the given UUID.
///
/// See `media_engine_exists` for Notes.
#[rustler::nif]
fn registry_exists<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    registry_uuid: Term<'a>,
) -> Term<'a> {
    let mut state = match resource.0.try_lock() {
        Err(_) => return (atoms::error(), atoms::lock_fail()).encode(env),
        Ok(guard) => guard,
    };

    match state.get_registry(registry_uuid) {
        None => (atoms::ok(), false).encode(env),
        Some(_r) => (atoms::ok(), true).encode(env),
    }
}

//
// PRIVATE
//

fn gen_uuid() -> String {
    Uuid::new_v4().to_hyphenated().to_string()
}

fn parse_init_opts<'a>(env: Env<'a>, opts: Term<'a>) -> Result<Config, Atom> {
    if !opts.is_map() {
        return Err(atoms::invalid_configuration());
    };

    let ice_servers = match opts.map_get(atoms::ice_servers().to_term(env)) {
        Err(_) => return Err(atoms::invalid_configuration()),
        Ok(servers) => servers.decode().unwrap(),
    };

    let config = Config::new(ice_servers);

    Ok(config)
}
