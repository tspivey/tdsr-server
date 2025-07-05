#![windows_subsystem = "windows"]

use anyhow::{Context, Result};
use native_dialog::{DialogBuilder, MessageLevel};
use std::{
	env,
	io::{BufRead, BufReader},
	net::{TcpListener, TcpStream},
	process, thread,
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

#[derive(Debug)]
enum UserEvent {
	MenuEvent(tray_icon::menu::MenuEvent),
}

const DEFAULT_PORT: u16 = 64111;
const MAX_LINE_LENGTH: usize = 10000;

fn main() -> Result<()> {
	let port = parse_port_from_args().unwrap_or(DEFAULT_PORT);
	let listener = TcpListener::bind(("0.0.0.0", port)).with_context(|| format!("Unable to bind to port {port}"))?;
	thread::spawn(move || {
		for connection in listener.incoming() {
			thread::spawn(move || {
				if let Err(e) = connection.context("Failed to accept connection").and_then(handle_connection) {
					show_error(&format!("Connection error: {e:?}"));
				}
			});
		}
	});
	run_tray_application()
}

fn parse_port_from_args() -> Option<u16> {
	env::args().nth(1)?.parse().ok()
}

fn run_tray_application() -> Result<()> {
	let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
	let proxy = event_loop.create_proxy();
	MenuEvent::set_event_handler(Some(move |event| {
		let _ = proxy.send_event(UserEvent::MenuEvent(event));
	}));
	let mut tray_icon = None;
	let tray_menu = Menu::new();
	let quit_item = MenuItem::new("&Quit", true, None);
	tray_menu.append(&quit_item)?;
	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Wait;
		match event {
			Event::NewEvents(tao::event::StartCause::Init) => {
				tray_icon = TrayIconBuilder::new()
					.with_menu(Box::new(tray_menu.clone()))
					.with_tooltip("TDSR Server")
					.build()
					.context("Failed to create tray icon")
					.map_err(|e| {
						show_error(&format!("Tray icon error: {e:?}"));
						process::exit(1);
					})
					.ok();
			}
			Event::UserEvent(UserEvent::MenuEvent(event)) => {
				if event.id == quit_item.id() {
					tray_icon.take();
					*control_flow = ControlFlow::Exit;
				}
			}
			_ => {}
		}
	});
}

fn handle_connection(connection: TcpStream) -> Result<()> {
	let mut reader = BufReader::new(connection);
	let mut line = String::new();
	let mut tts = Tts::default().context("Failed to initialize TTS")?;
	while reader.read_line(&mut line)? > 0 {
		let trimmed_line = line.trim_end_matches(['\n', '\r']);
		if let Some((command, arg)) = trimmed_line.split_at_checked(1) {
			process_command(command, arg, &mut tts);
		}
		line.clear();
	}
	Ok(())
}

fn process_command(command: &str, arg: &str, tts: &mut Tts) {
	match command {
		"s" | "l" if !arg.is_empty() => {
			let cleaned_text = arg.replace('\u{23CE}', " ");
			let options = Options::new(MAX_LINE_LENGTH).break_words(false);
			for chunk in wrap(&cleaned_text, &options) {
				speak(&chunk, tts);
			}
		}
		"x" => stop_speaking(tts),
		_ => {}
	}
}

fn speak(text: &str, tts: &mut Tts) {
	if let Err(e) = tts.speak(text, false) {
		show_error(&format!("Failed to speak: {e:?}"));
	}
}

fn stop_speaking(tts: &mut Tts) {
	if let Err(e) = tts.stop() {
		show_error(&format!("Failed to stop speaking: {e:?}"));
	}
}

fn show_error(message: &str) {
	let _ = DialogBuilder::message()
		.set_title("TDSR Server Error")
		.set_level(MessageLevel::Error)
		.set_text(message)
		.alert();
}
