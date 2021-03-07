
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
    pub node: bool,
}
impl Sender {
    pub fn connect(&mut self, cert: String) {
        let mut builder = SslConnector::builder(SslMethod::tls_client()).unwrap();
        // FIX: Hostname mismach maybe becouse im testing on localhost, have to fix later 
        builder.set_verify(SslVerifyMode::NONE);
        builder.set_certificate_chain_file(cert.clone()).unwrap();
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
            let mut data = [0 as u8; 8192];
            let mut nodes = Vec::new();
            for dst in self.dst_hosts.iter() {
                let handshake = Handshake {node_load: 0, node_memory: 0, success: false, transport_token: None};
                let sender_stream = TcpStream::connect(dst).unwrap();
                let mut stream = connector.connect(dst, sender_stream).unwrap();
                
                let handshake_serialized = serde_json::to_string(&handshake).unwrap();
                stream.write(handshake_serialized.as_bytes()).unwrap();

                //TODO Timeout on read
                let handshake_serialized = match stream.read(&mut data) {
                    Ok(size) => from_utf8(&data[0..size]).unwrap(),
                    Err(_)   => continue 
                };

                let handshake: Handshake = serde_json::from_str(&handshake_serialized).unwrap();
                let node = Node {handshake: handshake, node: dst.to_string()};
                nodes.push(node);
            }
            let node = balancer::select_node(nodes);
            let sender_stream = TcpStream::connect(&node.node).unwrap();
            self.stream = Some(connector.connect(&node.node, sender_stream).unwrap());
            let handshake_serialized = serde_json::to_string(&node.handshake).unwrap();
            if let Some(ref mut stream) = self.stream {
                stream.write(handshake_serialized.as_bytes()).unwrap();
            }
        }
    }
    
    pub fn write(&mut self, mut message: String) {
        if let Some(ref mut stream) = self.stream {
            message.push_str("\n");
            stream.write(message.as_bytes()).unwrap();
        } else {
            println!("Sending stream not inicialized");
        }
        
    }
}
