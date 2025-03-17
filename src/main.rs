#![windows_subsystem = "windows"]
use native_dialog::{MessageDialog, MessageType};
use std::{
    error::Error,
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    process,
    sync::{Arc, Mutex},
    thread,
};
use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder},
};
use textwrap::{Options, wrap};
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};
use tts::Tts;

type TtsRef = Arc<Mutex<Result<Tts, tts::Error>>>;

enum UserEvent {
    MenuEvent(tray_icon::menu::MenuEvent),
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:64111").map_err(|error| {
        show_error(&format!("Unable to bind: {:?}", error));
        process::exit(1);
    })?;
    let tts = Arc::new(Mutex::new(Tts::default()));
    thread::spawn(move || {
        for connection in listener.incoming() {
            let tts = Arc::clone(&tts);
            thread::spawn(move || {
                let connection = match connection {
                    Ok(conn) => conn,
                    Err(e) => {
                        show_error(&format!("Failed to accept connection: {}", e));
                        return;
                    }
                };
                if let Err(e) = handle_connection(connection, &tts) {
                    show_error(&format!("Error handling connection: {}", e));
                }
            });
        }
    });
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));
    let mut tray_icon = None;
    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("&Quit", true, None);
    tray_menu.append(&quit_i)?;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(tao::event::StartCause::Init) => {
                tray_icon = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(tray_menu.clone()))
                        .with_tooltip("TDSR Server")
                        .build()
                        .unwrap(),
                );
            }
            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                if event.id == quit_i.id() {
                    tray_icon.take();
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
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
            if let Err(e) = tts.speak(text, false) {
                show_error(&format!("Failed to speak: {}", e));
            }
        }
    }
}

fn stop_speaking(tts: &TtsRef) {
    if let Ok(mut tts) = tts.lock() {
        if let Ok(tts) = tts.as_mut() {
            if let Err(e) = tts.stop() {
                show_error(&format!("Failed to stop speaking: {}", e));
            }
        }
    }
}

fn show_error(message: &str) {
    let _ = MessageDialog::new()
        .set_title("Error")
        .set_type(MessageType::Error)
        .set_text(message)
        .show_alert();
}
