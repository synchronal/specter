rustler::atoms! {
    ok,
    error,

    // errors
    answer_error,
    invalid_atom,
    invalid_json,
    invalid_local_description,
    invalid_remote_description,
    lock_fail,
    not_found,
    offer_error,

    // config
    ice_servers,
    invalid_configuration,

    webrtc_error,

    // send
    current_local_description,
    data_channel_created,
    local_description,
    peer_connection_closed,
    peer_connection_error,
    peer_connection_ready,
    pending_local_description,
    set_local_description,
    set_remote_description,

    answer,
    offer,
}
