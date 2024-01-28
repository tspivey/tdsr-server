use std::net::{TcpListener, TcpStream};
use std::process;
mod nvda;
use crate::nvda::*;
use std::io::{BufRead, BufReader};
use std::thread;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:64111").unwrap_or_else(|error| {
        eprintln!("Unable to bind: {:?}", error);
        process::exit(1);
    });
    for con in listener.incoming() {
        thread::spawn(|| {
            let res = handle_connection(con.unwrap());
            if let Err(e) = res {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
    let s = String::from("This is a test of a very long string that will take a while to say");
    speak(&s);
    stop_speaking();
    //println!("{}", s);
}

fn handle_connection(con: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(con);
    loop {
        let mut line = String::new();
        let len = reader.read_line(&mut line)?;
        if len == 0 {
            return Ok(());
        }
        // Find a better way to do this
        if line.ends_with("\n") {
            line.pop();
        }
        if line.ends_with("\r") {
            line.pop();
        }
        let mut chars = line.chars();
        let command = match chars.next() {
            Some(c) => String::from(c),
            None => {
                continue;
            }
        };
        let arg: String = chars.collect();
        process_command(&command, &arg);
    }
}

fn process_command(command: &str, arg: &str) {
    match command {
        "s" | "l" => {
            if arg != "" {
                speak(arg);
            }
        }
        "x" => {
            stop_speaking();
        }
        _ => (),
    }
}
