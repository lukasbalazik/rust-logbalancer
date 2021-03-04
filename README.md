# rust-logbalancer
Logbalancer in rust

Under development

## installation

add to Cargo.toml:
```
[dependencies]
logbalancer = {  git = "https://github.com/lukasbalazik123/rust-logbalancer"  }
```

## usage
### Sender
```
extern crate logbalancer;

use logbalancer::{LogBalancer, Settings};

fn main() {
    let mut dst_hosts = Vec::new();
    dst_hosts.push(String::from("HOST:PORT of node"));
    dst_hosts.push(String::from("HOST:PORT of node"));

    let logbalancer = LogBalancer {
        settings: Settings::sender_settings(String::from("<HOST:PORT for listening>"), dst_hosts))
    };
    logbalancer.start();
}
```
### Node
```
extern crate logbalancer;

use logbalancer::{LogBalancer, Settings};

fn main() {
    let logbalancer = LogBalancer {
        settings: Settings::node_settings(String::from("<HOST:PORT for listening>"), String::from("<HOST:PORT for log destination>")))
    };
    logbalancer.start();
}
```
