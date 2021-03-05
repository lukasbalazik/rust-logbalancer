use openssl::ssl::{SslVerifyMode, SslMethod, SslStream, SslConnector};
use std::net::TcpStream;
use std::io::{Read, Write};

pub struct Sender {
    pub dst_hosts: Vec<String>,
    pub stream: Option<SslStream<TcpStream>>
}
impl Sender {
    pub fn connect(&mut self, cert: String, node: bool) {
        let mut connector = SslConnector::builder(SslMethod::tls_client()).unwrap();
        // FIX: Hostname mismach maybe becouse im testing on localhost, have to fix later 
        connector.set_verify(SslVerifyMode::NONE);
        connector.set_ca_file(cert.clone()).unwrap();
        let connector = connector.build();
        

        if node == true {
            match self.dst_hosts.last() {
                Some(dst) => {
                    let stream = TcpStream::connect(dst).unwrap();
                    self.stream = Some(connector.connect(dst, stream).unwrap());
                },
                _ => panic!("No senders available"),
            };
        } else {
            for dst in self.dst_hosts.iter() {
                let sender_stream = TcpStream::connect(dst).unwrap();
                // TODO: call function which will get node which is available
                // if vysledok callu true:
                self.stream = Some(connector.connect(dst, sender_stream).unwrap());
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
