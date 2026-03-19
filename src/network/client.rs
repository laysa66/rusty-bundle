use std::io::{Read, Write};
use std::net::TcpStream;
use crate::protobuf::bundle_proto::Bundle as ProtobufBundle;
use crate::routing::model::Node;

/// Connects to a peer node and sends a message
pub fn send_message(address: &str, message: &str) {
    // Try to connect to the peer
    match TcpStream::connect(address) {
        Ok(mut stream) => {
            println!("[Client] Connected to {}", address);
            stream.write_all(message.as_bytes())
                .expect("Failed to send message");
            println!("[Client] Message sent: {}", message);

            // Read the response from the server
            let mut buffer = [0; 1024];
            stream.read(&mut buffer)
                .expect("Failed to read response");
            let response = String::from_utf8_lossy(&buffer);
            println!("[Client] Server response: {}", response);
        }
        Err(e) => {
            println!("[Client] Could not connect to {}: {}", address, e);
        }
    }
}

pub fn send_bundle(bundle: ProtobufBundle, peers: Vec<Node>) {

    // convert the bundle into the protobuf generated bundle
    //serialization of the protobuf to JSON string using protobuf-json-mapping
    let payload = match protobuf_json_mapping::print_to_string(&bundle) {
        Ok(json) => json.into_bytes(),
        Err(e) => {
            eprintln!("send_bundle failed to serialize bundle {} : {}", bundle.id, e);
            return;
        }
    };

    //iterate over eachh peer that the routing engine decided on
    for peer in peers {
        
        //build the peer address from the ip and port
        let address = format!("{}:{}", peer.address, peer.port);

        //Open a direct TCP connection to the peer
        let mut stream = match TcpStream::connect(address) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("send_bundle TCP connect to {} failed: {}", address, e);
                return;
            }
        };

        //sending with length prefix to let the receiver know exactly how many bytes to read
        let len = payload.len() as u32;
        if let Err(e) = stream
            .write_all(&len.to_be_bytes()) //writing the entire buffer to the tcp stream
            .and_then(|_| stream.write_all(&payload)) // this only runs if the previous is Ok
        {
            eprintln!("send_bundle failed to write to {}: {}", address, e);
            return;
        }

        //waiting for the ack peers
        let mut ack = [0u8; 4]; //buffer allocation
        match stream.read_exact(&mut ack) { //reads exactly 4 bytes from tcp stream into the ack buffer
            Ok(_) if &ack == b"ack\n" => {}
            Ok(_) => eprintln!("[send_bundle] unexpected ack from {}: {:?}", address, ack),
            Err(e) => eprintln!("[send_bundle] failed to read ack from {}: {}", address, e),
        }

    }

}