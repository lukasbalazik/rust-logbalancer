extern crate serde;
extern crate serde_json;
extern crate sys_info;

use openssl::ssl::{SslMethod, SslAcceptor, SslStream, SslFiletype};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::io::{Read, Write};
use std::str::from_utf8;
use std::thread;

mod balancer;
mod messager;
mod sender;
mod settings;

use messager::Messager;
use sender::Sender;

pub use settings::Settings;
pub use settings::Handshake;


pub struct LogBalancer {
    pub settings: Settings,
    pub transport_token_function: Option<fn(String) -> String>,
    pub certificate_chain_file: String,
    pub ca_file: String,
    pub private_key_file: String,
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

    fn sender_initialize(settings: Settings, ca_file: &String) -> Sender {
        let mut sender = Sender {dst_hosts: settings.dst_hosts, stream: None, node: settings.node, selected_node: None };
        loop {
            sender.connect(ca_file.clone());
            if settings.node != true && sender.check_node() != true {
                println!("Node is not initiliazed or did not end successfuly reconnecting");
                continue
            }
            break
        }
        sender
    }

    fn handle_client(mut receiver: SslStream<TcpStream>, settings: Settings, ca_file: String) {
        let mut data = [0 as u8; 8192];

        let mut handshake = settings.handshake.clone();
        let mut sender = LogBalancer::sender_initialize(settings.clone(), &ca_file);
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
                handshake = LogBalancer::handshake_initialize(message);
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
                    while sender.write(line.to_string()) != true {
                        sender = LogBalancer::sender_initialize(settings.clone(), &ca_file);
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

        
        for stream in listener.incoming() {
            let receiver = match stream {
                Ok(stream) => stream,
                Err(e) => panic!("Error: {}", e),
            };

            let acceptor = acceptor.clone();
            let set = self.settings.clone();
            let sender_cert = self.ca_file.clone();

            thread::spawn(move || {
                let receiver = acceptor.accept(receiver).unwrap();
                LogBalancer::handle_client(receiver, set, sender_cert)
            });
        }
        drop(listener);
    }
}
