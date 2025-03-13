use std::{
    error::Error,
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    process,
    sync::{Arc, Mutex},
    thread,
};
use textwrap::{Options, wrap};
use tts::Tts;

type TtsRef = Arc<Mutex<Result<Tts, tts::Error>>>;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:64111").map_err(|error| {
        eprintln!("Unable to bind: {:?}", error);
        process::exit(1);
    })?;
    let tts = Arc::new(Mutex::new(Tts::default()));
    for connection in listener.incoming() {
        let tts = Arc::clone(&tts);
        thread::spawn(move || {
            let connection = match connection {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                    return;
                }
            };
            if let Err(e) = handle_connection(connection, &tts) {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
    Ok(())
}

fn handle_connection(connection: TcpStream, tts: &TtsRef) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(connection);
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        let trimmed_line = line.trim_end_matches(&['\n', '\r'][..]);
        if let Some((command, arg)) = trimmed_line.chars().next().map(|c| (c, &trimmed_line[1..])) {
            process_command(&command.to_string(), arg, tts);
        }
        line.clear();
    }
    Ok(())
}

fn process_command(command: &str, arg: &str, tts: &TtsRef) {
    match command {
        "s" | "l" if !arg.is_empty() => {
            let options = Options::new(10000).break_words(false);
            for chunk in wrap(arg, options) {
                speak(&chunk, tts);
            }
        }
        "x" => stop_speaking(tts),
        _ => (),
    }
}

fn speak(text: &str, tts: &TtsRef) {
    if let Ok(mut tts) = tts.lock() {
        if let Ok(tts) = tts.as_mut() {
            if let Err(e) = tts.speak(text, true) {
                eprintln!("Failed to speak: {}", e);
            }
        }
    }
}

fn stop_speaking(tts: &TtsRef) {
    if let Ok(mut tts) = tts.lock() {
        if let Ok(tts) = tts.as_mut() {
            if let Err(e) = tts.stop() {
                eprintln!("Failed to stop speaking: {}", e);
            }
        }
    }
}
