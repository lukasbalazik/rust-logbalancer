
use std::net::TcpStream;
use std::io::{Read, Write};

pub struct Sender {
    pub dst_hosts: Vec<String>,
    pub stream: Option<TcpStream>
}
impl Sender {
    pub fn connect(&mut self, node: bool) {
        if node == true {
            self.stream = match self.dst_hosts.last() {
                Some(dst) => Some(TcpStream::connect(dst).unwrap()),
                _ => panic!("No senders available"),
            };
        } else {
            for dst in self.dst_hosts.iter() {
                let sender_stream = TcpStream::connect(dst).unwrap();
                // TODO: call function which will get node which is available
                // if vysledok callu true:
                self.stream = Some(sender_stream);
            }
        }
    }
    
    pub fn write(&self, mut message: String) {
        if let Some(ref stream) = self.stream {
            let mut s = stream;
            message.push_str("\n");
            s.write(message.as_bytes()).unwrap();
        } else {
            println!("Sending stream not inicialized");
        }
        
    }
}
