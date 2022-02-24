use crate::atoms;
use crate::specter_config::SpecterConfig;
use rustler::{Atom, Encoder, Env, ResourceArc, Term};
use std::sync::Mutex;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;

pub struct State {
    config: RTCConfiguration,
}

impl State {}

pub struct StateResource(Mutex<State>);

#[rustler::nif(name = "__init__")]
fn init<'a>(env: Env<'a>, opts: Term<'a>) -> Term<'a> {
    let rtc_config = match parse_init_opts(env, opts) {
        Err(error) => return (atoms::error(), error).encode(env),
        Ok(rtc_config) => rtc_config,
    };

    let state = State { config: rtc_config };
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
