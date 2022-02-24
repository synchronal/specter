use crate::atoms;
use crate::specter_config::SpecterConfig;
use rustler::{Atom, Encoder, Env, ResourceArc, Term};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use webrtc::api::media_engine::MediaEngine;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;

pub struct State {
    config: RTCConfiguration,
    media_engines: HashMap<String, MediaEngine>,
}

impl State {
    pub(crate) fn add_media_engine(&mut self, uuid: &String, engine: MediaEngine) -> &mut State {
        self.media_engines.insert(uuid.clone(), engine);
        self
    }
}

pub struct StateResource(Mutex<State>);

#[rustler::nif(name = "__init__")]
fn init<'a>(env: Env<'a>, opts: Term<'a>) -> Term<'a> {
    let rtc_config = match parse_init_opts(env, opts) {
        Err(error) => return (atoms::error(), error).encode(env),
        Ok(rtc_config) => rtc_config,
    };

    let state = State {
        config: rtc_config,
        media_engines: HashMap::new(),
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

#[rustler::nif]
fn new_media_engine<'a>(
    env: Env<'a>,
    resource: ResourceArc<StateResource>,
) -> Result<Term<'a>, Atom> {
    let mut state = match resource.0.try_lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    // Create a MediaEngine object to configure the default supported codecs
    let mut m = MediaEngine::default();
    match m.register_default_codecs() {
        Err(_error) => return Err(atoms::webrtc_error()),
        Ok(term) => term,
    }

    let engine_id = Uuid::new_v4().to_hyphenated().to_string();
    state.add_media_engine(&engine_id, m);
    Ok(engine_id.encode(env))
}

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
