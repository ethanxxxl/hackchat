use core::time;
use std::env;
use std::io::{self, stdout, Read, Write};
use std::net::{Shutdown, TcpStream, SocketAddr};
use std::sync::mpsc;
use std::thread;

use termion::raw::IntoRawMode;

use packet::Packet;

fn main() -> std::io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = stdout();

    let mut server_addr = String::new();

    let args: Vec<String> = env::args().collect();

    if args.contains(&"--debug".to_string()) {
        server_addr = "127.0.0.1:4000".into();
    } else {
        loop {
            server_addr.clear();
            print!("enter the server address: ");
            stdout.flush()?;
            stdin.read_line(&mut server_addr)?;
            server_addr.pop(); // newline character at end causes problems

            if let Err(_) = server_addr.parse::<SocketAddr>() {
                println!("you entered: {}, incorrect format", server_addr);
                continue;
            } else {
                break;
            }
        }
    }

    let mut name = String::new();
    print!("enter your name: ");
    stdout.flush()?;
    stdin.read_line(&mut name)?;
    name.pop();
    println!("I got: {} for your name", name);

    let mut stdout = stdout.into_raw_mode().unwrap();

    let (command_tx, command_rx) = mpsc::channel();
    let (message_tx, message_rx) = mpsc::channel();

    // message loop, sends/recieves messges from the server.
    thread::spawn(move || {
        let mut stream = TcpStream::connect(server_addr).expect("error connecting to server");
        stream.set_nonblocking(true).unwrap();

        Packet::Connect(name).send(&mut stream).unwrap();

        loop {
            // get command from user input stream
            if let Ok(cmd) = command_rx.try_recv() {
                Packet::Message(cmd).send(&mut stream).unwrap();
            }

            // recieve and handle packets from the server
            while let Ok(pkt) = Packet::recv(&mut stream) {
                match pkt {
                    Packet::Message(msg) => message_tx.send(msg).unwrap(),
                    _ => (),
                }
            }
        }

        stream.shutdown(Shutdown::Both).unwrap();
    });

    // user interface loop. Gets user input and sends it to the server.
    let mut stdin = termion::async_stdin().bytes();
    let mut command = String::new();
    let mut chat_history = Vec::new();
    loop {
        stdout.flush().unwrap();
        thread::sleep(time::Duration::from_millis(20));

        use termion::{clear, cursor};
        write!(stdout, "{}", clear::All).unwrap();

        let (x, y) = termion::terminal_size().unwrap();

        // create horizontal line separating user input from messages
        write!(stdout, "{}", cursor::Goto(1, y - 1)).unwrap();
        for _ in 0..x {
            write!(stdout, "⎯").unwrap();
        }

        // post new messages
        if let Ok(msg) = message_rx.try_recv() {
            chat_history.push(msg);
        }

        // write chat messages
        for (i, msg) in chat_history.iter().rev().enumerate() {
            if i as u16 > (y - 2) {
                break;
            }

            write!(stdout, "{}{msg}", cursor::Goto(1, y - 2 - i as u16)).unwrap();
        }

        // move cursor to input box
        write!(stdout, "{}> {}", cursor::Goto(1, y), command,).unwrap();

        // get next byte for the command
        let mut is_newline = false;
        if let Some(Ok(b)) = stdin.next() {
            // set flag for finished command
            match b as char {
                '\n' | '\x0D' => is_newline = true,
                '\x08' | '\x7F' => {
                    command.pop();
                    write!(stdout, "{}", cursor::Left(1)).unwrap();
                } // backspace
                '\r' => (),
                '\x03' => break, // control-C
                _ => command.push(b as char),
            }
        }

        // guard statement for the following match statement
        if !is_newline {
            continue;
        }

        // deal with input
        match command.as_str() {
            "/help" => {
                // TODO write out help output
            }
            "/refresh" => {
                // TODO force refresh
            }
            "/quit" => break,
            _ => command_tx.send(command.to_owned()).unwrap(),
        }

        command.clear();
    }

    Ok(())
}
