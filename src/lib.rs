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
    fn handle_client(mut receiver: SslStream<TcpStream>, settings: Settings, sender_cert: String) {
        let mut data = [0 as u8; 8192];

        let mut sender = Sender {dst_hosts: settings.dst_hosts, stream: None, node: settings.node};
        let mut messager = Messager { penultimate_last_line: String::from(""), complete: true };
        let mut handshake = Handshake { transport_token: None, success: false, node_load: 0, node_memory: 0 };

        sender.connect(sender_cert);

        loop {
            let message = match receiver.read(&mut data) {
                Ok(size) => from_utf8(&data[0..size]).unwrap(),
                Err(_)   => break,
            };

            if settings.node == true && handshake.success != true {
                // TODO this to separate function so users can program own handshake
                handshake = match serde_json::from_str(&message) {
                    Ok(h) => h,
                    Err(_) => break, 
                };
                if handshake.success != true {
                    let enriched_handshake  = balancer::get_system_info(handshake.clone());
                    let enriched_handshake_serialized = serde_json::to_string(&enriched_handshake).unwrap();
                    receiver.write(enriched_handshake_serialized.as_bytes()).unwrap();
                }
                continue;
            }

            match message {
                "" => break,
                _  => {
                    let corrected_message = messager.corrector(message);
                    let last_message: usize = corrected_message.lines().count();

                    let mut lines = corrected_message.lines();
                    let mut counter: usize = 1;
                    
                    while let Some(line) = lines.next() {
                        if counter == last_message && messager.complete != true {
                            messager.set_penultimate_last_line(String::from(line));
                        } else {
                                sender.write(line.to_string());
                        }
                        counter += 1
                    }
                },
            };
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
