use crate::atoms;
use crate::codec_capability::RtpCodecCapability;
use crate::config::Config;
use crate::peer_connection;
use crate::util::gen_uuid;
use rustler::types::pid::Pid;
use rustler::{Atom, Encoder, Env, ResourceArc, Term};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;
use webrtc::api::interceptor_registry as interceptor;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::{APIBuilder, API};
use webrtc::interceptor::registry::Registry;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;

// The resource which will be wrapped in an ResourceArc and returned to
// Elixir as a reference.
pub struct Ref(pub(crate) Arc<Mutex<State>>);

pub struct State {
    pub config: Config,
    pub pid: Pid,

    apis: HashMap<String, Arc<API>>,
    media_engines: HashMap<String, MediaEngine>,
    peer_connections: HashMap<String, Sender<peer_connection::Msg>>,
    registries: HashMap<String, Registry>,
    local_static_sample_tracks: HashMap<String, Arc<TrackLocalStaticSample>>,
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
            local_static_sample_tracks: HashMap::new(),
        }
    }

    //***** API

    pub(crate) fn add_api(&mut self, uuid: &str, api: API) -> &mut State {
        self.apis.insert(uuid.to_owned(), Arc::new(api));
        self
    }

    pub(crate) fn get_api(&self, uuid: Term) -> Option<&Arc<API>> {
        let id: &String = &uuid.decode().unwrap();
        self.apis.get(id)
    }

    //***** MediaEngine

    pub(crate) fn add_media_engine(&mut self, uuid: &str, engine: MediaEngine) -> &mut State {
        self.media_engines.insert(uuid.to_owned(), engine);
        self
    }

    pub(crate) fn get_media_engine(&mut self, uuid: Term) -> Option<&MediaEngine> {
        // This could stand some error handling. The match implementation
        // fails with "creates a temporary which is freed while still in use."
        let id: &String = &uuid.decode().unwrap();
        self.media_engines.get(id)
    }

    pub(crate) fn get_media_engine_mut(&mut self, uuid: Term) -> Option<&mut MediaEngine> {
        let id: &String = &uuid.decode().unwrap();
        self.media_engines.get_mut(id)
    }

    pub(crate) fn remove_media_engine(&mut self, uuid: Term) -> Option<MediaEngine> {
        let id: &String = &uuid.decode().unwrap();
        self.media_engines.remove(id)
    }

    //***** RTCPeerConnection

    pub(crate) fn add_peer_connection(
        &mut self,
        uuid: &str,
        pc: Sender<peer_connection::Msg>,
    ) -> &mut State {
        self.peer_connections.insert(uuid.to_owned(), pc);
        self
    }

    pub(crate) fn get_peer_connection(&self, uuid: Term) -> Option<&Sender<peer_connection::Msg>> {
        let id: &String = &uuid.decode().unwrap();
        self.peer_connections.get(id)
    }

    pub(crate) fn remove_peer_connection(
        &mut self,
        uuid: Term,
    ) -> Option<Sender<peer_connection::Msg>> {
        let id: &String = &uuid.decode().unwrap();
        self.peer_connections.remove(id)
    }

    //***** Registry

    pub(crate) fn add_registry(&mut self, uuid: &str, registry: Registry) -> &mut State {
        self.registries.insert(uuid.to_owned(), registry);
        self
    }

    pub(crate) fn get_registry(&mut self, uuid: Term) -> Option<&Registry> {
        let id: &String = &uuid.decode().unwrap();
        self.registries.get(id)
    }

    pub(crate) fn remove_registry(&mut self, uuid: Term) -> Option<Registry> {
        let id: &String = &uuid.decode().unwrap();
        self.registries.remove(id)
    }

    //***** Track
    pub(crate) fn add_track_local_static_sample(
        &mut self,
        uuid: &str,
        track: Arc<TrackLocalStaticSample>,
    ) -> &mut State {
        self.local_static_sample_tracks
            .insert(uuid.to_owned(), track);
        self
    }

    pub(crate) fn get_track_local_static_sample(
        &mut self,
        uuid: &String,
    ) -> Option<&Arc<TrackLocalStaticSample>> {
        self.local_static_sample_tracks.get(uuid)
    }
}

pub fn load(env: Env) -> bool {
    rustler::resource!(Ref, env);
    true
}

/// Initialize the NIF, returning a reference to Elixir that can be
/// passed back into the NIF to retrieve or alter state.
#[rustler::nif(name = "__init__")]
fn init<'a>(env: Env<'a>, opts: Term<'a>) -> Term<'a> {
    let config = match Config::parse(env, opts) {
        Err(error) => return (atoms::error(), error).encode(env),
        Ok(config) => config,
    };

    let state = State::new(config, env.pid());
    let resource = ResourceArc::new(Ref(Arc::new(Mutex::new(state))));

    (atoms::ok(), resource).encode(env)
}

#[rustler::nif(name = "config")]
fn get_config(env: Env, resource: ResourceArc<Ref>) -> Result<Term, Atom> {
    let state = match resource.0.lock() {
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
fn new_media_engine(resource: ResourceArc<Ref>) -> Result<String, Atom> {
    let mut state = match resource.0.lock() {
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
    Ok(engine_id)
}

/// Create an intercepter registry.
///
/// Open questions:
/// - What the heck is an intercepter registry?
/// - How is it used later?
#[rustler::nif]
fn new_registry(resource: ResourceArc<Ref>, media_engine_uuid: Term) -> Result<String, Atom> {
    let mut state = match resource.0.lock() {
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
    Ok(registry_id)
}

/// Create a new API. This is directly used when creating RTCPeerConnections.
///
/// Open questions:
/// - This is used to create RTCPeerConnections. Is it used for anything else?
/// - I thought we needed to create a new registry for each PC. Does that
///   mean we need to create a new one of these for each PC?
#[rustler::nif]
fn new_api<'a>(
    resource: ResourceArc<Ref>,
    media_engine_uuid: Term<'a>,
    registry_uuid: Term<'a>,
) -> Result<String, Atom> {
    let mut state = match resource.0.lock() {
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
    Ok(api_id)
}

#[rustler::nif]
fn new_track_local_static_sample<'a>(
    resource: ResourceArc<Ref>,
    codec: Term<'a>,
    id: Term<'a>,
    stream_id: Term<'a>,
) -> Result<String, Atom> {
    let mut state = match resource.0.lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    let codec: RtpCodecCapability = codec.decode().unwrap();
    let track = TrackLocalStaticSample::new(
        RTCRtpCodecCapability::from(codec),
        id.decode().unwrap(),
        stream_id.decode().unwrap(),
    );
    let track_id = gen_uuid();
    state.add_track_local_static_sample(&track_id, Arc::new(track));
    Ok(track_id)
}

/// Returns true or false depending on whether the State hashmap owns a MediaEngine
/// for the given UUID.
///
/// Some entities need to take ownership of a given MediaEngine, for example when
/// creating an API. After that happens, the MediaEngine will no longer be available
/// in the State hashmap.
#[rustler::nif]
fn media_engine_exists(resource: ResourceArc<Ref>, media_engine_uuid: Term) -> Result<bool, Atom> {
    let mut state = match resource.0.lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    match state.get_media_engine(media_engine_uuid) {
        None => Ok(false),
        Some(_m) => Ok(true),
    }
}
///
/// Returns true or false depending on whether the State hashmap owns an RTCPeerConnection
/// for the given UUID.
#[rustler::nif]
fn peer_connection_exists(resource: ResourceArc<Ref>, pc_uuid: Term) -> Result<bool, Atom> {
    let state = match resource.0.lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    match state.get_peer_connection(pc_uuid) {
        None => Ok(false),
        Some(_m) => Ok(true),
    }
}

/// Returns true or false depending on whether the State hashmap owns a Registry
/// for the given UUID.
///
/// See `media_engine_exists` for Notes.
#[rustler::nif]
fn registry_exists(resource: ResourceArc<Ref>, registry_uuid: Term) -> Result<bool, Atom> {
    let mut state = match resource.0.lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    match state.get_registry(registry_uuid) {
        None => Ok(false),
        Some(_r) => Ok(true),
    }
}
