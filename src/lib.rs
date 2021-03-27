extern crate serde;
extern crate serde_json;
extern crate sys_info;

use openssl::ssl::{SslMethod, SslAcceptor, SslStream, SslFiletype};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::io::{Read, Write};
use std::str::from_utf8;
use std::thread;
use std::time;

pub mod balancer;
mod messager;
mod sender;
mod settings;

use messager::Messager;
use sender::Sender;

pub use settings::Settings;
pub use settings::Handshake;

#[derive(Clone)]
pub struct LogBalancer {
    pub settings: Settings,
    pub custom_handshake_initialize: Option<fn(&str) -> Handshake>,
    pub custom_update_dst_hosts: Option<fn(String) -> Vec<String>>,
    pub certificate_chain_file: String,
    pub private_key_file: String,
    pub ca_file: Option<String>,
}
impl LogBalancer {

    fn handshake_initialize(message: &str) -> Handshake {
        let mut handshake: Handshake = match serde_json::from_str(&message) {
            Ok(h) => h,
            Err(_) => panic!("Cant read handshake from message"), 
        };
        if handshake.initialized != true {
            handshake = balancer::get_system_info(handshake.clone());
            handshake.initialized = true; 
        } 
        handshake
    }

    fn sender_initialize(settings: Settings, ca_file: Option<String>) -> Sender {
        let mut sender = Sender {dst_hosts: settings.dst_hosts, stream: None, node: settings.node, selected_node: None, handshake: settings.handshake };
        let sleep_time = time::Duration::from_millis(10000);
        loop {
            sender.connect(ca_file.clone());
            if settings.node != true && sender.check_node() != true {
                println!("Node is not initiliazed or did not end successfuly reconnecting");
                thread::sleep(sleep_time);
                continue
            }
            break
        }
        sender
    }

    fn handle_client(mut receiver: SslStream<TcpStream>, self_struct: LogBalancer) {
        let mut data = [0 as u8; 8192];
        let ca_file = self_struct.ca_file;
        let settings = self_struct.settings;

        let mut handshake = settings.handshake.clone();
        let mut sender = LogBalancer::sender_initialize(settings.clone(), ca_file.clone());
        let mut messager = Messager { penultimate_last_line: String::from(""), complete: true };

        loop {
            let message = match receiver.read(&mut data) {
                Ok(size) => from_utf8(&data[0..size]).unwrap(),
                Err(_)   => break,
            };

            if message.eq("") {
                break;
            }

            if settings.node == true && handshake.success != true {
                
                if let Some(ref handshake_initialize) = self_struct.custom_handshake_initialize {
                    handshake = handshake_initialize(message);
                }
                if handshake.initialized != true {
                    break;
                }
                
                let handshake_serialized = serde_json::to_string(&handshake).unwrap();
                receiver.write(handshake_serialized.as_bytes()).unwrap();
                
                continue;
            }

            let corrected_message = messager.corrector(message);
            let last_message: usize = corrected_message.lines().count();

            let mut lines = corrected_message.lines();
            let mut counter: usize = 1;
            
            while let Some(line) = lines.next() {
                if counter == last_message && messager.complete != true {
                    messager.set_penultimate_last_line(String::from(line));
                } else {
                    if sender.write(line.to_string()) != true {
                        sender = LogBalancer::sender_initialize(settings.clone(), ca_file.clone());
                    }
                }
                counter += 1
            }
        }
    }

    pub fn start(&mut self) {
        let mut acceptor = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server()).unwrap();
        acceptor.set_private_key_file(self.private_key_file.clone(), SslFiletype::PEM).unwrap();
        acceptor.set_certificate_chain_file(self.certificate_chain_file.clone()).unwrap();
        acceptor.check_private_key().unwrap();

        let acceptor = Arc::new(acceptor.build());
        let listener = TcpListener::bind(&self.settings.listen_host).unwrap();

        if self.custom_handshake_initialize.is_none() {
            self.custom_handshake_initialize = Some(LogBalancer::handshake_initialize);
        }
        
        for stream in listener.incoming() {
            if let Some(ref mut update_dst_hosts) = self.custom_update_dst_hosts {
                if let Some(ref mut token) = self.settings.handshake.transport_token {
                    self.settings.dst_hosts = update_dst_hosts(token.to_string());
                }
            }
            let receiver = match stream {
                Ok(stream) => stream,
                Err(e) => panic!("Error: {}", e),
            };

            let acceptor = acceptor.clone();

            let self_struct = self.clone();

            thread::spawn(move || {
                let receiver = acceptor.accept(receiver).unwrap();
                LogBalancer::handle_client(receiver, self_struct)
            });
        }
        drop(listener);
    }
}



