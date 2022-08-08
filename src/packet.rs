use std::net::TcpStream;
use std::io::{Read, Write};

#[derive(Debug)]
pub enum Packet {
    Error,
    Connect(String),
    Message(String),
    Channel(u32),
    //ChatHistory(Vec<String>),
}

impl Packet {
    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        // determine the header id for the packet
        use Packet::*;
        let header: u32 = match self {
            Error => 0,
            Connect(_) => 1,
            Message(_) => 2,
            Channel(_) => 3,
            //ChatHistory(_) => 2,
        };

        // convert it to an array of 4 bytes.
        let header = header.to_be_bytes();

        // serialize the body of the packet
        let body: Vec<u8> = match self {
            Error => Vec::new(),
            Connect(username) => username.to_owned().into_bytes(),
            Message(msg) => msg.to_owned().into_bytes(),
            Channel(n) => n.to_be_bytes().to_vec(),
            //ChatHistory(hst) => hst.concat().into_bytes(),
        };

        let body_len = (body.len() as u32).to_be_bytes();

        // the packet format is as follows:
        // [HeaderID; 4] [BodyLen; 4] [Body; ...]
        let mut packet = Vec::new();
        packet.extend_from_slice(&header);
        packet.extend_from_slice(&body_len);
        packet.extend(body);

        // push the packet into the stream
        if let Err(e) = stream.write(packet.as_slice()) {
            return Err(e)
        }

        if let Err(e) = stream.flush() {
            return Err(e)
        }

        Ok(())
    }

    pub fn recv(stream: &mut TcpStream) -> std::io::Result<Packet> {
        // read heading and body length from stream
        let mut buf = [0 as u8; 8];
        match stream.read(&mut buf) {
            Ok(n) if n == 8 => (),
            Err(e) => {
                return Err(e);
            },
            _ => {
                return Err(std::io::ErrorKind::InvalidData.into())
            }
        };

        let (header, body_len) = buf.split_at(4);
        let header = u32::from_be_bytes(header.try_into().unwrap());
        let body_len = u32::from_be_bytes(body_len.try_into().unwrap());

        // get rest of packet data;
        let mut buf = vec![0; body_len as usize];
        if let Err(e) = stream.read_exact(buf.as_mut_slice()) {
            return Err(e)
        };

        use Packet::*;
        let packet = match header {
            1 => Connect(String::from_utf8(buf).unwrap()),
            2 => Message(String::from_utf8(buf).unwrap()),
            3 => Channel(u32::from_be_bytes(buf.try_into().unwrap())),
            //4 => ChatHistory( ??? )
            _ => Error,
        };

        Ok(packet)
    }
}
