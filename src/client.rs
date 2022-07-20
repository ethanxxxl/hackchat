use std::collections::TryReserveError;
use std::net::{TcpStream, IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, Shutdown};
use std::io::{self, Read, Write};
use std::sync::mpsc;
use std::thread;

fn main() -> std::io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut server_addr = String::new();
    //loop {
    //    server_addr.clear();
    //    print!("enter the server address: ");
    //    stdout.flush()?;
    //    stdin.read_line(&mut server_addr)?;
    //    server_addr.pop(); // newline character at end causes problems

    //    if let Err(_) = server_addr.to_socket_addrs() {
    //        println!("you entered: {}, incorrect format", server_addr);
    //        continue;
    //    }

    //    if server_addr.to_socket_addrs().unwrap().count() > 0 {
    //        break;
    //    } else {
    //        println!("you entered: {}, incorrect format", server_addr);
    //    }

    //}

    server_addr = "127.0.0.1:4000".to_string();

    //let mut name = String::new();
    //print!("enter your name: ");
    //stdout.flush()?;
    //stdin.read_line(&mut name)?;
    //name.pop();

    //let (tx, rx) = mpsc::channel();

    let mut stream = TcpStream::connect(server_addr).expect("error connecting to server");
    stream.set_nonblocking(true)?;

    let mut message = String::new();
    loop {
        print!(">> ");
        stdout.flush()?;
        stdin.read_line(&mut message)?;
        //message.pop(); // get rid of annoying newline

        // get new messages from server
        if message.eq("/refresh\n") {
            let mut inbox = Vec::new();
            let mut buf = [0 as u8; 32];
            while let Ok(bytes_read) = stream.read(&mut buf) {
                inbox.extend_from_slice(&buf[..bytes_read]);
            }

            println!("{}", String::from_utf8(inbox).unwrap());

        } else if message.eq("/quit\n") {
            break;
        // send message to server
        } else if message.len() > 0 {
            stream.write(message.as_bytes())?;
            stream.flush()?;
        }

        message.clear();
    }

    stream.shutdown(Shutdown::Both)?;

    Ok(())
}
