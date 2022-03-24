use crate::atoms;
use rustler::types::elixir_struct;
use rustler::{Atom, Encoder, Env, Term};
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;

#[derive(Clone, Debug)]
pub struct Config {
    pub ice_servers: Vec<String>,
}

impl Config {
    pub fn parse<'a>(env: Env<'a>, opts: Term<'a>) -> Result<Config, Atom> {
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

    pub fn new(ice_servers: Vec<String>) -> Self {
        Config { ice_servers }
    }
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

impl From<Config> for RTCConfiguration {
    fn from(config: Config) -> Self {
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
        let config = elixir_struct::make_ex_struct(env, "Elixir.Specter.Config").unwrap();
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
