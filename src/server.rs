use std::net::{TcpListener, TcpStream, Ipv4Addr, SocketAddrV4, SocketAddr};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::io::{Read, Write};

struct ClientInfo {
    username: String,
    stream: TcpStream,
    address: SocketAddr,
    outbox: Vec<(String, String)>,
}

impl ClientInfo {
    fn new(stream: TcpStream) -> Self {
        Self {
            username: String::new(),
            stream,
            address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0,0,0,0), 0)),
            outbox: Vec::new(),
        }
    }
}


fn main() -> std::io::Result<()> {
    let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 4000);

    let listener = TcpListener::bind(socket)?;
    let socket = listener.local_addr().unwrap();
    println!("listening for connections on {:?}...", socket);

    // look for incomming connections
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                tx.send(stream).unwrap();
            } else {
                panic!("ERROR: failed to connect to client. {:?}", stream);
            }
        }
    });

    // check for incomming messages, and update all clients.
    let mut clients: HashMap<SocketAddr, ClientInfo> = HashMap::new();
    let mut messages: Vec<(String, String)> = Vec::new();
    loop {
        // add any new connections
        let mut new_connection: Option<SocketAddr> = None;
        if let Ok(stream) = rx.try_recv() {
            let client_sock = stream.peer_addr().unwrap();
            new_connection = Some(client_sock);

            stream.set_nonblocking(true);
            let mut client = ClientInfo::new(stream);
            client.outbox.push(("Server".to_string(), "Connection established!\n".to_string()));

            clients.insert(client_sock, client);
            println!("new conection from {:?}", client_sock);
        }

        // get incoming messages
        for (ip, client) in clients.iter_mut() {
            let mut new_msg = Vec::new();

            // fill new_msg with the recieved message
            let mut buf = [0 as u8; 32];
            while let Ok(bytes_read) = client.stream.read(&mut buf) {
                new_msg.extend_from_slice(&buf[..bytes_read]);
            }

            if new_msg.len() > 0 {
                //TODO do some processing on buf to figure out what the client actually wants.
                // for now, just send out all its contents.
                let new_msg = String::from_utf8(new_msg).unwrap();
                println!("recieved \"{}\" from {:?}", new_msg, ip);
                messages.push((client.username.clone(), new_msg.to_string()));
            }
        }

        // send outbound messages
        for (ip, client) in clients.iter_mut() {
            for msg in client.outbox.iter() {
                let msg = format!("{}: {}", msg.0, msg.1);
                client.stream.write(msg.as_bytes())?;
                client.stream.flush()?;
                println!("sending \"{}\" to {:?}", msg, ip);
            }
            client.outbox.clear();


            for msg in messages.iter() {
                let msg = format!("{}: {}", msg.0, msg.1);
                client.stream.write(msg.as_bytes())?;
                client.stream.flush()?;
                println!("sending \"{}\" to {:?}", msg, ip);
            }
        }
        messages.clear();

    }

    Ok(())
}
