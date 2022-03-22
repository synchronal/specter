use crate::atoms;
use crate::config::Config;
use crate::task;
use log::trace;
use rustler::types::pid::Pid;
use rustler::{Atom, Encoder, Env, ResourceArc, Term};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{channel, Sender};
use uuid::Uuid;
use webrtc::api::interceptor_registry as interceptor;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::{APIBuilder, API};
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
// use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::offer_answer_options::{RTCAnswerOptions, RTCOfferOptions};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

pub struct State {
    apis: HashMap<String, Arc<API>>,
    config: Config,
    media_engines: HashMap<String, MediaEngine>,
    peer_connections: HashMap<String, Sender<RTCPCMsg>>,
    pid: Pid,
    registries: HashMap<String, Registry>,
}

pub enum RTCPCMsg {
    CreateAnswer(String, Option<RTCAnswerOptions>),
    CreateDataChannel(String, String),
    CreateOffer(String, Option<RTCOfferOptions>),
    SetRemoteDescription(String, RTCSessionDescription),
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
        let id: &String = &uuid.clone().decode().unwrap();
        self.apis.get(id)
    }

    //***** MediaEngine

    pub(crate) fn add_media_engine(&mut self, uuid: &String, engine: MediaEngine) -> &mut State {
        self.media_engines.insert(uuid.clone(), engine);
        self
    }

    pub(crate) fn get_media_engine<'a>(&mut self, uuid: Term<'a>) -> Option<&MediaEngine> {
        // This could stand some error handling. The match implementation
        // fails with "creates a temporary which is freed while still in use."
        let id: &String = &uuid.clone().decode().unwrap();
        self.media_engines.get(id)
    }

    pub(crate) fn get_media_engine_mut<'a>(&mut self, uuid: Term<'a>) -> Option<&mut MediaEngine> {
        let id: &String = &uuid.clone().decode().unwrap();
        self.media_engines.get_mut(id)
    }

    pub(crate) fn remove_media_engine<'a>(&mut self, uuid: Term<'a>) -> Option<MediaEngine> {
        let id: &String = &uuid.clone().decode().unwrap();
        self.media_engines.remove(id)
    }

    //***** RTCPeerConnection

    pub(crate) fn add_peer_connection(
        &mut self,
        uuid: &String,
        pc: Sender<RTCPCMsg>,
    ) -> &mut State {
        self.peer_connections.insert(uuid.clone(), pc);
        self
    }

    pub(crate) fn get_peer_connection<'a>(&self, uuid: Term<'a>) -> Option<&Sender<RTCPCMsg>> {
        let id: &String = &uuid.clone().decode().unwrap();
        self.peer_connections.get(id)
    }

    pub(crate) fn remove_peer_connection<'a>(
        &mut self,
        uuid: Term<'a>,
    ) -> Option<Sender<RTCPCMsg>> {
        let id: &String = &uuid.clone().decode().unwrap();
        self.peer_connections.remove(id)
    }

    //***** Registry

    pub(crate) fn add_registry(&mut self, uuid: &String, registry: Registry) -> &mut State {
        self.registries.insert(uuid.clone(), registry);
        self
    }

    pub(crate) fn get_registry<'a>(&mut self, uuid: Term<'a>) -> Option<&Registry> {
        let id: &String = &uuid.clone().decode().unwrap();
        self.registries.get(id)
    }

    pub(crate) fn remove_registry<'a>(&mut self, uuid: Term<'a>) -> Option<Registry> {
        let id: &String = &uuid.clone().decode().unwrap();
        self.registries.remove(id)
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
fn new_media_engine<'a>(resource: ResourceArc<Ref>) -> Result<String, Atom> {
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
fn new_registry<'a>(
    resource: ResourceArc<Ref>,
    media_engine_uuid: Term<'a>,
) -> Result<String, Atom> {
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

/// Create a new RTCPeerConnection.
///
/// Open questions:
/// - Once this is initialized, how does it run in a thread that doesn't conflict
///   with the Erlang scheduler?
#[rustler::nif]
fn new_peer_connection<'a>(resource: ResourceArc<Ref>, api_uuid: Term<'a>) -> Result<String, Atom> {
    let api = {
        let state_ref = resource.0.lock().unwrap();
        match state_ref.get_api(api_uuid) {
            None => return Err(atoms::not_found()),
            Some(a) => Arc::clone(a),
        }
    };

    let uuid = gen_uuid();
    spawn_rtc_peer_connection(resource, api, uuid.clone());

    Ok(uuid)
}

/// Close an RTCPeerConnection. This pops out the Sender for the task holding the peer connection,
/// causing it to go out of scope. That causes a `None` to come out of the Sender's recv block.
#[rustler::nif]
fn close_peer_connection<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
) -> Term<'a> {
    let mut state = match resource.0.lock() {
        Err(_) => return (atoms::error(), atoms::lock_fail()).encode(env),
        Ok(guard) => guard,
    };

    let _tx = match state.remove_peer_connection(pc_uuid) {
        None => return (atoms::error(), atoms::not_found()).encode(env),
        Some(tx) => tx,
    };

    (atoms::ok()).encode(env)
}

/// Returns true or false depending on whether the State hashmap owns a MediaEngine
/// for the given UUID.
///
/// Some entities need to take ownership of a given MediaEngine, for example when
/// creating an API. After that happens, the MediaEngine will no longer be available
/// in the State hashmap.
#[rustler::nif]
fn media_engine_exists<'a>(
    resource: ResourceArc<Ref>,
    media_engine_uuid: Term<'a>,
) -> Result<bool, Atom> {
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
fn peer_connection_exists<'a>(resource: ResourceArc<Ref>, pc_uuid: Term<'a>) -> Result<bool, Atom> {
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
fn registry_exists<'a>(resource: ResourceArc<Ref>, registry_uuid: Term<'a>) -> Result<bool, Atom> {
    let mut state = match resource.0.lock() {
        Err(_) => return Err(atoms::lock_fail()),
        Ok(guard) => guard,
    };

    match state.get_registry(registry_uuid) {
        None => Ok(false),
        Some(_r) => Ok(true),
    }
}

#[rustler::nif]
fn create_answer<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
    voice_activity_detection: bool,
) -> Term<'a> {
    let state = match resource.0.lock() {
        Err(_) => return (atoms::error(), atoms::lock_fail()).encode(env),
        Ok(guard) => guard,
    };

    let tx = match state.get_peer_connection(pc_uuid) {
        None => return (atoms::error(), atoms::not_found()).encode(env),
        Some(tx) => tx.clone(),
    };

    let answer_opts = RTCAnswerOptions {
        voice_activity_detection,
    };

    let uuid = pc_uuid.clone().decode().unwrap();

    task::spawn(async move {
        match tx
            .send(RTCPCMsg::CreateAnswer(uuid, Some(answer_opts)))
            .await
        {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
}

/// Implemented without options to facilitate the creation of offers with ufrag and pwd,
/// so that these offers may be given to other peer connections without errors.
#[rustler::nif]
fn create_data_channel<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
    label: String,
) -> Term<'a> {
    let state = match resource.0.lock() {
        Err(_) => return (atoms::error(), atoms::lock_fail()).encode(env),
        Ok(guard) => guard,
    };

    let tx = match state.get_peer_connection(pc_uuid) {
        None => return (atoms::error(), atoms::not_found()).encode(env),
        Some(tx) => tx.clone(),
    };

    let uuid = pc_uuid.clone().decode().unwrap();

    task::spawn(async move {
        match tx.send(RTCPCMsg::CreateDataChannel(uuid, label)).await {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
}

/// Create an offer. Note that media tracks and data channels must be given to these
/// peer connection prior to calling this.
#[rustler::nif]
fn create_offer<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
    voice_activity_detection: bool,
    ice_restart: bool,
) -> Term<'a> {
    let state = match resource.0.lock() {
        Err(_) => return (atoms::error(), atoms::lock_fail()).encode(env),
        Ok(guard) => guard,
    };

    let tx = match state.get_peer_connection(pc_uuid) {
        None => return (atoms::error(), atoms::not_found()).encode(env),
        Some(tx) => tx.clone(),
    };

    let offer_opts = RTCOfferOptions {
        ice_restart,
        voice_activity_detection,
    };

    let uuid = pc_uuid.clone().decode().unwrap();

    task::spawn(async move {
        match tx.send(RTCPCMsg::CreateOffer(uuid, Some(offer_opts))).await {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
}

/// Receives an offer or an answer from a remote entity, and sets it on an
/// existing RTCPeerConnection.
#[rustler::nif]
fn set_remote_description<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
    sdp: String,
) -> Term<'a> {
    let state = match resource.0.lock() {
        Err(_) => return (atoms::error(), atoms::lock_fail()).encode(env),
        Ok(guard) => guard,
    };

    // let sdp_type = match sdp_type {
    //     t if t == atoms::answer() => RTCSdpType::Answer,
    //     t if t == atoms::offer() => RTCSdpType::Offer,
    //     _ => return (atoms::error(), atoms::invalid_atom()).encode(env),
    // };
    // let session_description = RTCSessionDescription {
    //     sdp_type,
    //     sdp,
    //     ..Default::default()
    // };

    let session_description = match serde_json::from_str::<RTCSessionDescription>(&sdp) {
        Err(_) => return (atoms::error(), atoms::invalid_json()).encode(env),
        Ok(s) => s,
    };

    let tx = match state.get_peer_connection(pc_uuid) {
        None => return (atoms::error(), atoms::not_found()).encode(env),
        Some(tx) => tx.clone(),
    };

    let uuid = pc_uuid.clone().decode().unwrap();

    task::spawn(async move {
        match tx
            .send(RTCPCMsg::SetRemoteDescription(uuid, session_description))
            .await
        {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
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

fn spawn_rtc_peer_connection(resource: ResourceArc<Ref>, api: Arc<API>, uuid: String) {
    task::spawn(async move {
        let mut msg_env = rustler::env::OwnedEnv::new();

        let pc = {
            let state = resource.0.lock().unwrap();
            let rtc_config = RTCConfiguration::from(&state.config.clone());
            api.new_peer_connection(rtc_config)
        };

        let pc = match pc.await {
            Err(_) => {
                let state = resource.0.lock().unwrap();
                msg_env.send_and_clear(&state.pid, |env| {
                    (atoms::peer_connection_error(), uuid).encode(env)
                });
                return ();
            }
            Ok(pc) => Arc::new(pc),
        };

        let mut rx = {
            let (tx, rx) = channel::<RTCPCMsg>(1000);
            let mut state = resource.0.lock().unwrap();
            state.add_peer_connection(&uuid, tx);
            msg_env.send_and_clear(&state.pid, |env| {
                (atoms::peer_connection_ready(), &uuid).encode(env)
            });

            rx
        };

        // Block on messages being received on the channel for this peer connection.
        // When all senders go out of scope, the receiver will receive `None` and
        // break out of the loop.
        loop {
            match rx.recv().await {
                Some(RTCPCMsg::CreateAnswer(uuid, opts)) => {
                    let lock = pc.clone();
                    let resp = match lock.create_answer(opts).await {
                        Err(err) => Err((uuid, err)),
                        Ok(answer) => Ok((uuid, answer)),
                    };

                    let state = resource.0.lock().unwrap();
                    msg_env.send_and_clear(&state.pid, |env| match resp {
                        Err((uuid, err)) => {
                            (atoms::answer_error(), uuid, err.to_string()).encode(env)
                        }
                        Ok((uuid, answer)) => (
                            atoms::answer(),
                            uuid,
                            serde_json::to_string(&answer).unwrap(),
                        )
                            .encode(env),
                    });
                }
                Some(RTCPCMsg::CreateDataChannel(uuid, label)) => {
                    let lock = pc.clone();
                    let resp = match lock.create_data_channel(&label, None).await {
                        Err(err) => Err((uuid, err)),
                        Ok(data_channel) => Ok((uuid, data_channel)),
                    };

                    let state = resource.0.lock().unwrap();
                    msg_env.send_and_clear(&state.pid, |env| match resp {
                        Err((uuid, err)) => {
                            (atoms::offer_error(), uuid, err.to_string()).encode(env)
                        }
                        Ok((uuid, _data_channel)) => {
                            (atoms::data_channel_created(), uuid).encode(env)
                        }
                    });
                }
                Some(RTCPCMsg::CreateOffer(uuid, opts)) => {
                    let lock = pc.clone();
                    let resp = match lock.create_offer(opts).await {
                        Err(err) => Err((uuid, err)),
                        Ok(offer) => Ok((uuid, offer)),
                    };

                    let state = resource.0.lock().unwrap();
                    msg_env.send_and_clear(&state.pid, |env| match resp {
                        Err((uuid, err)) => {
                            (atoms::offer_error(), uuid, err.to_string()).encode(env)
                        }
                        Ok((uuid, offer)) => {
                            (atoms::offer(), uuid, serde_json::to_string(&offer).unwrap())
                                .encode(env)
                        }
                    });
                }
                Some(RTCPCMsg::SetRemoteDescription(uuid, session)) => {
                    let lock = pc.clone();
                    let resp = match lock.set_remote_description(session).await {
                        Err(err) => Err((uuid, err)),
                        Ok(_) => Ok(uuid),
                    };

                    let state = resource.0.lock().unwrap();
                    msg_env.send_and_clear(&state.pid, |env| match resp {
                        Err((uuid, err)) => {
                            (atoms::invalid_remote_description(), uuid, err.to_string()).encode(env)
                        }
                        Ok(uuid) => {
                            (atoms::ok(), uuid, atoms::set_remote_description()).encode(env)
                        }
                    });
                }
                None => break,
            }
        }

        let state = resource.0.lock().unwrap();
        msg_env.send_and_clear(&state.pid, |env| {
            (atoms::peer_connection_closed(), uuid).encode(env)
        });
    });
}
