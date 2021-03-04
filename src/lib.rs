extern crate serde;
extern crate serde_json;

use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::str::from_utf8;

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
    pub settings: Settings
}
impl LogBalancer {
    fn handle_client(mut stream: TcpStream, settings: Settings) {
        let mut data = [0 as u8; 8192];

        let mut sender = Sender {dst_hosts: settings.dst_hosts, stream: None};
        let mut messager = Messager{ penultimate_last_line: String::from(""), complete: true };

        sender.connect(true);


        loop {
            let message = match stream.read(&mut data) {
                Ok(size) => {
                    let message = match from_utf8(&data[0..size]) {
                        Ok(v)  => v,
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                    message
                },
                Err(_)   => {
                    println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
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
        
        stream.shutdown(Shutdown::Both).unwrap();
    }

    pub fn start(&self) {
        let listener = TcpListener::bind(&self.settings.listen_host).unwrap();

        println!("Server listening on {}", self.settings.listen_host);
        for stream in listener.incoming() {
            let receiver = match stream {
                Ok(stream) => stream,
                Err(e) => panic!("Error: {}", e),
            };
            println!("New connection: {}", receiver.peer_addr().unwrap());
            let set = self.settings.clone();
            thread::spawn(move|| {
                LogBalancer::handle_client(receiver, set)
            });                 
        }
        drop(listener);
    }
}
