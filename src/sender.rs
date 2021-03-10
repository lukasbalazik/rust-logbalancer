
use openssl::ssl::{SslVerifyMode, SslMethod, SslStream, SslConnector};
use std::net::TcpStream;
use std::io::{Read, Write};
use std::str::from_utf8;

use crate::balancer;
use crate::balancer::Node;
use crate::settings::Handshake;


pub struct Sender {
    pub dst_hosts: Vec<String>,
    pub stream: Option<SslStream<TcpStream>>,
    pub selected_node: Option<Node>,
    pub node: bool,
}
impl Sender {
    fn get_node_health(&self, connector: SslConnector, node: &str) -> Handshake {
        let mut data = [0 as u8; 8192];
        let handshake = Handshake {node_load: 0, node_memory: 0, initialized: false, success: false, transport_token: None};
        let sender_stream = TcpStream::connect(node).unwrap();
        let mut stream = connector.connect(node, sender_stream).unwrap();
        
        let handshake_serialized = serde_json::to_string(&handshake).unwrap();
        stream.write(handshake_serialized.as_bytes()).unwrap();

        //TODO Timeout on read
        let handshake_serialized = match stream.read(&mut data) {
            Ok(size) => from_utf8(&data[0..size]).unwrap(),
            Err(_)   => "" 
        };
        if handshake_serialized.eq("") {
            return handshake;
        }
        serde_json::from_str(&handshake_serialized).unwrap()
    }

    pub fn check_node(&mut self) -> bool {
        let mut data = [0 as u8; 8192];
        let mut handshake_serialized: String;
        if let Some(ref node) = self.selected_node {
            handshake_serialized = serde_json::to_string(&node.handshake).unwrap();
        } else {
            println!("Cant get selected_node");
            return false;
        }
        if let Some(ref mut stream) = self.stream {
            stream.write(handshake_serialized.as_bytes()).unwrap();
            handshake_serialized = match stream.read(&mut data) {
                Ok(size) => from_utf8(&data[0..size]).unwrap().to_string(),
                Err(_)   => panic!("Cant read output from node")
            };
            let handshake: Handshake = serde_json::from_str(&handshake_serialized).unwrap();
            if handshake.initialized != true || handshake.success != true {
                return false;
            }
            return true;
        }
        false
    }

    pub fn connect(&mut self, ca_file: Option<String>) {
        let mut builder = SslConnector::builder(SslMethod::tls_client()).unwrap();
        // FIX: Hostname mismach maybe becouse im testing on localhost, have to fix later 
        builder.set_verify(SslVerifyMode::NONE);
        
        if let Some(ref ca_file) = ca_file {
            builder.set_ca_file(ca_file.clone()).unwrap();
        }
        let connector = builder.build();
        

        if self.node == true {
            match self.dst_hosts.last() {
                Some(dst) => {
                    let stream = TcpStream::connect(dst).unwrap();
                    self.stream = Some(connector.connect(dst, stream).unwrap());
                },
                _ => panic!("No destination hosts available"),
            };
        } else {
            let mut nodes = Vec::new();
            for dst in self.dst_hosts.iter() {
                let handshake = self.get_node_health(connector.clone(), dst);
                if handshake.initialized != true {
                    println!("Handshake initialization failed");
                    continue;
                }
                let node = Node {handshake: handshake, node: dst.to_string()};
                nodes.push(node);
            }
            if nodes.len() == 0 {
                println!("Handshake did not success on any of destination nodes");
            } else {
                let node = balancer::select_node(nodes);
                let sender_stream = TcpStream::connect(&node.node).unwrap();
                self.stream = Some(connector.connect(&node.node, sender_stream).unwrap());
                self.selected_node = Some(node);
            }
        }
    }
    
    pub fn write(&mut self, mut message: String) -> bool {
        if let Some(ref mut stream) = self.stream {
            message.push_str("\n");
            stream.write(message.as_bytes()).unwrap();
            return true;
        } else {
            println!("Sending stream not inicialized");
        }
        false
    }
}
