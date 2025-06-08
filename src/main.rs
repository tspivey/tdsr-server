#![windows_subsystem = "windows"]

use anyhow::{Context, Result};
use native_dialog::{MessageDialog, MessageType};
use std::{
    env,
    io::BufRead,
    net::{TcpListener, TcpStream},
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

const DEFAULT_PORT: u16 = 64111;

#[derive(Debug)]
enum UserEvent {
    MenuEvent(MenuEvent),
}

fn main() -> Result<()> {
    let port = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    start_server(port)?;
    run_tray_app()
}

fn start_server(port: u16) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .with_context(|| format!("Failed to bind to port {port}"))?;
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(|| handle_connection(stream).unwrap_or_else(log_error));
                }
                Err(e) => log_error(e.into()),
            }
        }
    });
    Ok(())
}

fn run_tray_app() -> Result<()> {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));
    let quit_item = MenuItem::new("&Quit", true, None);
    let menu = Menu::new();
    menu.append(&quit_item)?;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(tao::event::StartCause::Init) => {
                TrayIconBuilder::new()
                    .with_menu(Box::new(menu.clone()))
                    .with_tooltip("TDSR Server")
                    .build()
                    .map_err(|e| log_error(e.into()))
                    .ok();
            }
            Event::UserEvent(UserEvent::MenuEvent(event)) if event.id == quit_item.id() => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn handle_connection(stream: TcpStream) -> Result<()> {
    let reader = std::io::BufReader::new(stream);
    let mut tts = Tts::default().context("Failed to initialize TTS")?;
    for line in reader.lines() {
        let line = line.context("Failed to read line")?;
        let trimmed = line.trim();
        if let Some((cmd, args)) = trimmed.split_at_checked(1) {
            process_command(cmd, args, &mut tts)?;
        }
    }
    Ok(())
}

fn process_command(cmd: &str, args: &str, tts: &mut Tts) -> Result<()> {
    match cmd {
        "s" | "l" if !args.is_empty() => speak_text(args, tts),
        "x" => stop_speech(tts),
        _ => Ok(()),
    }
}

fn speak_text(text: &str, tts: &mut Tts) -> Result<()> {
    let options = Options::new(10000).break_words(false);
    for chunk in wrap(text, options) {
        tts.speak(&*chunk, false).context("TTS speak failed")?;
    }
    Ok(())
}

fn stop_speech(tts: &mut Tts) -> Result<()> {
    tts.stop().context("Failed to stop TTS")?;
    Ok(())
}

fn log_error(err: anyhow::Error) {
    let _ = MessageDialog::new()
        .set_title("TDSR Server Error")
        .set_type(MessageType::Error)
        .set_text(&format!("{err:#}"))
        .show_alert();
}
