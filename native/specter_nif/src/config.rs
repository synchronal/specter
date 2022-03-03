use crate::atoms;
use rustler::types::elixir_struct::make_ex_struct;
use rustler::{Encoder, Env, Term};
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;

#[derive(Debug)]
pub struct Config {
    pub ice_servers: Vec<String>,
}

impl From<&Config> for RTCConfiguration {
    fn from(config: &Config) -> Self {
        RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: config.ice_servers.clone(),
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

impl Encoder for Config {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let config = make_ex_struct(env, "Elixir.Specter.Config").unwrap();
        let mut ice_servers = Term::list_new_empty(env);

        ice_servers = self
            .ice_servers
            .iter()
            .fold(ice_servers, |ice_servers, server| {
                ice_servers.list_prepend(server.encode(env))
            })
            .list_reverse()
            .unwrap();

        config
            .map_put(atoms::ice_servers().to_term(env), ice_servers)
            .unwrap()
    }
}
