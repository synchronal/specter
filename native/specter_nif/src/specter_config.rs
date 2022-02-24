use crate::atoms;
use rustler::types::elixir_struct::make_ex_struct;
use rustler::{Encoder, Env, Term};
use webrtc::peer_connection::configuration::RTCConfiguration;

#[derive(Debug)]
pub struct SpecterConfig {
    pub ice_servers: Vec<String>,
}

impl SpecterConfig {
    pub fn from_rtc_configuration(rtc_config: &RTCConfiguration) -> SpecterConfig {
        let ice_servers = rtc_config.ice_servers.clone();
        let mut urls: Vec<String> = vec![];

        for ice_server in &ice_servers {
            for raw_url in &ice_server.urls {
                urls.push(raw_url.to_owned());
            }
        }

        SpecterConfig { ice_servers: urls }
    }
}

impl Encoder for SpecterConfig {
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
