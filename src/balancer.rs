use crate::settings::Handshake;

pub struct Node {
    pub handshake: Handshake,
    pub node: String,
}

pub fn get_system_info(mut handshake: Handshake) -> Handshake {
    let load = match sys_info::loadavg() {
        Ok(v) => v.five.round() as u8,
        Err(_) => panic!("Cannot get cpu load"),
    };

    let memory = match sys_info::mem_info() {
        Ok(v) => (((v.avail as f64)/(v.total as f64))*100.0).round() as u8,
        Err(_) => panic!("Cannot get available memory"),
    };
    handshake.node_load = load;
    handshake.node_memory = memory;
    handshake
}

pub fn select_node(nodes: Vec<Node>) -> Node {
    let mut selected_node = Node {node: String::from(""), handshake:
        Handshake { transport_token: None, success: false, node_load:255, node_memory: 255 }
    };
    for node in nodes.iter() {
        println!("node: {}", node.node);
        println!("node_load: {}", node.handshake.node_load);
        println!("node_memory: {}", node.handshake.node_memory);
        if node.handshake.node_load < selected_node.handshake.node_load && node.handshake.node_memory > 5 {
            selected_node = Node {node: node.node.clone(), handshake: node.handshake.clone() };
        }
        
    }
    selected_node.handshake.success = true;
    selected_node
}
