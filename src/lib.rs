extern crate serde;
extern crate serde_json;

use openssl::ssl::{SslMethod, SslAcceptor, SslStream, SslFiletype};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::io::{Read, Write};
use std::str::from_utf8;
use std::thread;

mod settings;
mod sender;

use sender::Sender;

pub use settings::Settings;
pub use settings::Handshake;


struct Messager {
    penultimate_last_line: String,
    complete: bool,
}
impl Messager {
    fn correctmessage(&mut self, received: String) -> String {
        let mut _message: String = "".to_owned();
        if self.complete != true { 
            _message.push_str(&self.penultimate_last_line);
        }
        _message.push_str(&received);

        if _message.chars().last().unwrap() != '\n' {
            self.complete = false;
        } else {
            self.complete = true;
        }

        _message
    }

    fn set_penultimate_last_line(&mut self, line: String) {
        self.penultimate_last_line = line;
    }


}

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

        let mut sender = Sender {dst_hosts: settings.dst_hosts, stream: None};
        let mut messager = Messager{ penultimate_last_line: String::from(""), complete: true };

        sender.connect(sender_cert, true);

        loop {
            let message = match receiver.read(&mut data) {
                Ok(size) => {
                    let message = match from_utf8(&data[0..size]) {
                        Ok(v)  => v,
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                    message
                },
                Err(_)   => {
                    break
                }
            };

            match message {
                "" => break,
                _  => {
                    let corrected_message = messager.correctmessage(message.to_string());
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
        
        receiver.shutdown().unwrap();
    }


    pub fn start(&mut self) {
        let mut acceptor = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server()).unwrap();
        acceptor.set_private_key_file(self.private_key_file.clone(), SslFiletype::PEM).unwrap();
        acceptor.set_certificate_chain_file(self.certificate_chain_file.clone()).unwrap();
        acceptor.check_private_key().unwrap();

        if self.settings.node == true {
            acceptor.set_ca_file(self.ca_file.clone()).unwrap();
        }

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
