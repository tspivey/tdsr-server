mod nvda;
use crate::nvda::*;
use std::{
    error::Error,
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    process, thread,
};
use textwrap::wrap;

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
}

fn handle_connection(con: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(con);
    loop {
        let mut line = String::new();
        let len = reader.read_line(&mut line)?;
        if len == 0 {
            return Ok(());
        }
        line = line.trim_end_matches(&['\n', '\r'][..]).to_string();
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
            if !arg.is_empty() {
                let chunk_size = 10000;
                let chunks = wrap(arg, chunk_size);
                for chunk in chunks {
                    speak(&chunk);
                }
            }
        }
        "x" => {
            stop_speaking();
        }
        _ => (),
    }
}
