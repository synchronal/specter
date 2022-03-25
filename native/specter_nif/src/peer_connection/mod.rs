use crate::atoms;
use crate::state::Ref;
use crate::task;
use crate::util::gen_uuid;
use log::trace;
use rustler::{Atom, Encoder, Env, ResourceArc, Term};
use serde_json;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use webrtc::api::API;
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
// use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::offer_answer_options::{RTCAnswerOptions, RTCOfferOptions};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

pub enum Msg {
    AddIceCandidate(String, RTCIceCandidateInit),
    CreateAnswer(String, Option<RTCAnswerOptions>),
    CreateDataChannel(String, String),
    CreateOffer(String, Option<RTCOfferOptions>),
    GetCurrentLocalDescription(String),
    GetCurrentRemoteDescription(String),
    GetLocalDescription(String),
    GetPendingLocalDescription(String),
    GetPendingRemoteDescription(String),
    GetRemoteDescription(String),
    SetLocalDescription(String, RTCSessionDescription),
    SetRemoteDescription(String, RTCSessionDescription),
}

/// Create a new RTCPeerConnection.
///
/// Open questions:
/// - Once this is initialized, how does it run in a thread that doesn't conflict
///   with the Erlang scheduler?
#[rustler::nif(name = "new_peer_connection")]
fn new<'a>(resource: ResourceArc<Ref>, api_uuid: Term<'a>) -> Result<String, Atom> {
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
#[rustler::nif(name = "close_peer_connection")]
fn close<'a>(env: Env<'a>, resource: ResourceArc<Ref>, pc_uuid: Term<'a>) -> Term<'a> {
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

#[rustler::nif]
fn add_ice_candidate<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
    candidate: String,
) -> Term<'a> {
    let state = match resource.0.lock() {
        Err(_) => return (atoms::error(), atoms::lock_fail()).encode(env),
        Ok(guard) => guard,
    };

    let tx = match state.get_peer_connection(pc_uuid) {
        None => return (atoms::error(), atoms::not_found()).encode(env),
        Some(tx) => tx.clone(),
    };

    let ice_candidate = match serde_json::from_str::<RTCIceCandidateInit>(&candidate) {
        Err(_) => return (atoms::error(), atoms::invalid_json()).encode(env),
        Ok(s) => s,
    };

    let uuid = pc_uuid.clone().decode().unwrap();

    task::spawn(async move {
        match tx.send(Msg::AddIceCandidate(uuid, ice_candidate)).await {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
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
        match tx.send(Msg::CreateAnswer(uuid, Some(answer_opts))).await {
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
        match tx.send(Msg::CreateDataChannel(uuid, label)).await {
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
        match tx.send(Msg::CreateOffer(uuid, Some(offer_opts))).await {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
}

/// Note that this is nil until the peer connection has successfully negotiated its connection.
#[rustler::nif(name = "current_local_description")]
fn get_current_local_description<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
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
        match tx.send(Msg::GetCurrentLocalDescription(uuid)).await {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
}

/// Note that this is nil until the peer connection has successfully negotiated its connection.
#[rustler::nif(name = "current_remote_description")]
fn get_current_remote_description<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
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
        match tx.send(Msg::GetCurrentRemoteDescription(uuid)).await {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
}

/// Sends back either the current or pending session description.
#[rustler::nif(name = "local_description")]
fn get_local_description<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
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
        match tx.send(Msg::GetLocalDescription(uuid)).await {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
}

/// Sends back either the current or pending session description.
#[rustler::nif(name = "remote_description")]
fn get_remote_description<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
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
        match tx.send(Msg::GetRemoteDescription(uuid)).await {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
}

/// Note that this may be nil after ICE negotiates.
#[rustler::nif(name = "pending_local_description")]
fn get_pending_local_description<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
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
        match tx.send(Msg::GetPendingLocalDescription(uuid)).await {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
}

/// Note that this may be nil after ICE negotiates.
#[rustler::nif(name = "pending_remote_description")]
fn get_pending_remote_description<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
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
        match tx.send(Msg::GetPendingRemoteDescription(uuid)).await {
            Ok(_) => (),
            Err(_err) => trace!("send error"),
        }
    });

    (atoms::ok()).encode(env)
}

/// Receives an offer or an answer pertaining to a specific peer connection,
/// and sets it as the local session description.
#[rustler::nif]
fn set_local_description<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    pc_uuid: Term<'a>,
    sdp: String,
) -> Term<'a> {
    let state = match resource.0.lock() {
        Err(_) => return (atoms::error(), atoms::lock_fail()).encode(env),
        Ok(guard) => guard,
    };

    let tx = match state.get_peer_connection(pc_uuid) {
        None => return (atoms::error(), atoms::not_found()).encode(env),
        Some(tx) => tx.clone(),
    };

    let session_description = match serde_json::from_str::<RTCSessionDescription>(&sdp) {
        Err(_) => return (atoms::error(), atoms::invalid_json()).encode(env),
        Ok(s) => s,
    };

    let uuid = pc_uuid.clone().decode().unwrap();

    task::spawn(async move {
        match tx
            .send(Msg::SetLocalDescription(uuid, session_description))
            .await
        {
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

    let tx = match state.get_peer_connection(pc_uuid) {
        None => return (atoms::error(), atoms::not_found()).encode(env),
        Some(tx) => tx.clone(),
    };

    let session_description = match serde_json::from_str::<RTCSessionDescription>(&sdp) {
        Err(_) => return (atoms::error(), atoms::invalid_json()).encode(env),
        Ok(s) => s,
    };

    let uuid = pc_uuid.clone().decode().unwrap();

    task::spawn(async move {
        match tx
            .send(Msg::SetRemoteDescription(uuid, session_description))
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

fn spawn_rtc_peer_connection(resource: ResourceArc<Ref>, api: Arc<API>, uuid: String) {
    task::spawn(async move {
        let mut msg_env = rustler::env::OwnedEnv::new();

        let (pc, pid) = {
            let state = resource.0.lock().unwrap();
            let rtc_config = RTCConfiguration::from(&state.config.clone());
            (api.new_peer_connection(rtc_config), state.pid.clone())
        };

        let pc = match pc.await {
            Err(_) => {
                msg_env.send_and_clear(&pid, |env| {
                    (atoms::peer_connection_error(), &uuid).encode(env)
                });
                return ();
            }
            Ok(pc) => Arc::new(pc),
        };

        let mut rx = {
            let (tx, rx) = channel::<Msg>(1000);
            let mut state = resource.0.lock().unwrap();
            state.add_peer_connection(&uuid, tx);
            msg_env.send_and_clear(&state.pid, |env| {
                (atoms::peer_connection_ready(), &uuid).encode(env)
            });

            rx
        };

        // This does something with extra threads... ownership of String is moved,
        // but in a way where the compiler needs some lifetime clarity; cloning the
        // data until a cleaner way can be figured out.
        let pc_uuid = uuid.clone();
        pc.on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
            let uuid = pc_uuid.clone();
            Box::pin(async move {
                let mut msg_env = rustler::env::OwnedEnv::new();
                if let Some(c) = c {
                    let candidate = c.to_json().await.unwrap();
                    let json = serde_json::to_string(&candidate).unwrap();

                    msg_env.send_and_clear(&pid, |env| {
                        // (atoms::ice_candidate(), &uuid, json).encode(env)
                        (atoms::ice_candidate(), uuid, json).encode(env)
                    });
                }
            })
        }))
        .await;

        // Block on messages being received on the channel for this peer connection.
        // When all senders go out of scope, the receiver will receive `None` and
        // break out of the loop.
        loop {
            match rx.recv().await {
                Some(Msg::AddIceCandidate(uuid, candidate)) => {
                    let lock = pc.clone();
                    let resp = lock.add_ice_candidate(candidate).await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        Err(err) => (atoms::candidate_error(), uuid, err.to_string()).encode(env),
                        Ok(()) => (atoms::ok(), uuid, atoms::add_ice_candidate()).encode(env),
                    });
                }
                Some(Msg::CreateAnswer(uuid, opts)) => {
                    let lock = pc.clone();
                    let resp = lock.create_answer(opts).await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        Err(err) => (atoms::answer_error(), uuid, err.to_string()).encode(env),
                        Ok(answer) => (
                            atoms::answer(),
                            uuid,
                            serde_json::to_string(&answer).unwrap(),
                        )
                            .encode(env),
                    });
                }
                Some(Msg::CreateDataChannel(uuid, label)) => {
                    let lock = pc.clone();
                    let resp = lock.create_data_channel(&label, None).await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        Err(err) => (atoms::offer_error(), uuid, err.to_string()).encode(env),
                        Ok(_data_channel) => (atoms::data_channel_created(), uuid).encode(env),
                    });
                }
                Some(Msg::CreateOffer(uuid, opts)) => {
                    let lock = pc.clone();
                    let resp = lock.create_offer(opts).await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        Err(err) => (atoms::offer_error(), uuid, err.to_string()).encode(env),
                        Ok(offer) => (atoms::offer(), uuid, serde_json::to_string(&offer).unwrap())
                            .encode(env),
                    });
                }
                Some(Msg::GetCurrentLocalDescription(uuid)) => {
                    let lock = pc.clone();
                    let resp = lock.current_local_description().await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        None => (
                            atoms::current_local_description(),
                            uuid,
                            rustler::types::atom::nil(),
                        )
                            .encode(env),
                        Some(desc) => (
                            atoms::current_local_description(),
                            uuid,
                            serde_json::to_string(&desc).unwrap(),
                        )
                            .encode(env),
                    });
                }
                Some(Msg::GetLocalDescription(uuid)) => {
                    let lock = pc.clone();
                    let resp = lock.local_description().await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        None => (
                            atoms::local_description(),
                            uuid,
                            rustler::types::atom::nil(),
                        )
                            .encode(env),
                        Some(desc) => (
                            atoms::local_description(),
                            uuid,
                            serde_json::to_string(&desc).unwrap(),
                        )
                            .encode(env),
                    });
                }
                Some(Msg::GetPendingLocalDescription(uuid)) => {
                    let lock = pc.clone();
                    let resp = lock.pending_local_description().await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        None => (
                            atoms::pending_local_description(),
                            uuid,
                            rustler::types::atom::nil(),
                        )
                            .encode(env),
                        Some(desc) => (
                            atoms::pending_local_description(),
                            uuid,
                            serde_json::to_string(&desc).unwrap(),
                        )
                            .encode(env),
                    });
                }
                Some(Msg::GetCurrentRemoteDescription(uuid)) => {
                    let lock = pc.clone();
                    let resp = lock.current_remote_description().await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        None => (
                            atoms::current_remote_description(),
                            uuid,
                            rustler::types::atom::nil(),
                        )
                            .encode(env),
                        Some(desc) => (
                            atoms::current_remote_description(),
                            uuid,
                            serde_json::to_string(&desc).unwrap(),
                        )
                            .encode(env),
                    });
                }
                Some(Msg::GetRemoteDescription(uuid)) => {
                    let lock = pc.clone();
                    let resp = lock.remote_description().await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        None => (
                            atoms::remote_description(),
                            uuid,
                            rustler::types::atom::nil(),
                        )
                            .encode(env),
                        Some(desc) => (
                            atoms::remote_description(),
                            uuid,
                            serde_json::to_string(&desc).unwrap(),
                        )
                            .encode(env),
                    });
                }
                Some(Msg::GetPendingRemoteDescription(uuid)) => {
                    let lock = pc.clone();
                    let resp = lock.pending_remote_description().await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        None => (
                            atoms::pending_remote_description(),
                            uuid,
                            rustler::types::atom::nil(),
                        )
                            .encode(env),
                        Some(desc) => (
                            atoms::pending_remote_description(),
                            uuid,
                            serde_json::to_string(&desc).unwrap(),
                        )
                            .encode(env),
                    });
                }
                Some(Msg::SetLocalDescription(uuid, session)) => {
                    let lock = pc.clone();
                    let resp = lock.set_local_description(session).await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        Err(err) => {
                            (atoms::invalid_local_description(), uuid, err.to_string()).encode(env)
                        }
                        Ok(_) => (atoms::ok(), uuid, atoms::set_local_description()).encode(env),
                    });
                }
                Some(Msg::SetRemoteDescription(uuid, session)) => {
                    let lock = pc.clone();
                    let resp = lock.set_remote_description(session).await;

                    msg_env.send_and_clear(&pid, |env| match resp {
                        Err(err) => {
                            (atoms::invalid_remote_description(), uuid, err.to_string()).encode(env)
                        }
                        Ok(_) => (atoms::ok(), uuid, atoms::set_remote_description()).encode(env),
                    });
                }
                None => break,
            }
        }

        let state = resource.0.lock().unwrap();
        msg_env.send_and_clear(&state.pid, |env| {
            (atoms::peer_connection_closed(), &uuid).encode(env)
        });
    });
}
