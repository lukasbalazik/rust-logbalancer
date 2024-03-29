use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize, Clone)]
pub struct Handshake {
    pub transport_token: Option<String>,
    pub node_load: u8,
    pub node_memory: u8,
    pub success: bool,
    pub initialized: bool,
    pub additional: String,
}

#[derive(Clone)]
pub struct Settings {
    pub listen_host: String,
    pub node: bool,
    pub dst_hosts: Vec<String>,
    pub handshake: Handshake,
}

impl Settings {
    pub fn sender_settings(listen_host: String, dst_hosts: Vec<String>) -> Settings {
        let handshake = Handshake { node_load: 0, node_memory: 0, success: false, transport_token: None, initialized: false, additional: String::from("")};
        Settings {
            listen_host: listen_host,
            dst_hosts: dst_hosts,
            handshake: handshake,
            node: false,
        }
    }

    pub fn node_settings(listen_host: String, dst: String) -> Settings {
        let handshake = Handshake { node_load: 0, node_memory: 0, success: false, transport_token: None, initialized: false, additional: String::from("")};
        let mut vec = Vec::new();
        vec.push(dst);
        Settings {
            listen_host: listen_host,
            dst_hosts: vec,
            handshake: handshake,
            node: true,
        }
    }
}
