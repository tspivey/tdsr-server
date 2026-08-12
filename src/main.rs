#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![windows_subsystem = "windows"]

use std::{
	env, fs,
	io::{BufRead, BufReader},
	net::{TcpListener, TcpStream},
	process, thread,
};

use native_dialog::{DialogBuilder, MessageLevel};
use tao::{
	event::Event,
	event_loop::{ControlFlow, EventLoopBuilder},
};
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

fn main() {
	let port = parse_port_from_args().unwrap_or(DEFAULT_PORT);
	let letter_pitch_percent_change = read_letter_pitch_percent_change();
	let listener = match TcpListener::bind(("0.0.0.0", port)) {
		Ok(listener) => listener,
		Err(e) => {
			show_error(&format!("Unable to bind to port {port}: {e}"));
			return;
		}
	};
	thread::spawn(move || {
		for connection in listener.incoming() {
			thread::spawn(move || match connection {
				Ok(stream) => {
					if let Err(e) = handle_connection(stream, letter_pitch_percent_change) {
						show_error(&format!("Connection error: {e}"));
					}
				}
				Err(e) => show_error(&format!("Failed to accept connection: {e}")),
			});
		}
	});
	run_tray_application();
}

fn parse_port_from_args() -> Option<u16> {
	env::args().nth(1)?.parse().ok()
}

fn run_tray_application() {
	let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
	let proxy = event_loop.create_proxy();
	MenuEvent::set_event_handler(Some(move |event| {
		let _ = proxy.send_event(UserEvent::MenuEvent(event));
	}));
	let mut tray_icon = None;
	let tray_menu = Menu::new();
	let quit_item = MenuItem::new("&Quit", true, None);
	if let Err(e) = tray_menu.append(&quit_item) {
		show_error(&format!("Failed to build tray menu: {e:?}"));
		return;
	}
	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Wait;
		match event {
			Event::NewEvents(tao::event::StartCause::Init) => {
				tray_icon = match TrayIconBuilder::new()
					.with_menu(Box::new(tray_menu.clone()))
					.with_tooltip("TDSR Server")
					.build()
				{
					Ok(icon) => Some(icon),
					Err(e) => {
						show_error(&format!("Tray icon error: {e:?}"));
						process::exit(1);
					}
				};
			}
			Event::UserEvent(UserEvent::MenuEvent(event)) => {
				if event.id == quit_item.id() {
					if let Some(icon) = tray_icon.take() {
						drop(icon);
					}
					*control_flow = ControlFlow::Exit;
				}
			}
			_ => {}
		}
	});
}

fn handle_connection(connection: TcpStream, letter_pitch_percent_change: u32) -> Result<(), String> {
	let mut reader = BufReader::new(connection);
	let mut line = String::new();
	let mut tts = Tts::default().map_err(|e| format!("Failed to initialize TTS: {e:?}"))?;
	while reader.read_line(&mut line).map_err(|e| format!("Failed to read line: {e}"))? > 0 {
		let trimmed_line = line.trim_end_matches(['\n', '\r']);
		if let Some((command, arg)) = trimmed_line.split_at_checked(1) {
			process_command(command, arg, letter_pitch_percent_change, &mut tts);
		}
		line.clear();
	}
	Ok(())
}

fn process_command(command: &str, arg: &str, letter_pitch_percent_change: u32, tts: &mut Tts) {
	match command {
		"s" if !arg.is_empty() => {
			let cleaned_text = arg.replace('\u{23CE}', " ");
			speak_in_chunks(&cleaned_text, tts);
		}
		"l" if !arg.is_empty() => {
			let cleaned_text = arg.replace('\u{23CE}', " ");
			speak_as_letters(&cleaned_text, letter_pitch_percent_change, tts);
		}
		"x" => stop_speaking(tts),
		_ => {}
	}
}

fn read_letter_pitch_percent_change() -> u32 {
	env::current_exe()
		.ok()
		.and_then(|mut path| {
			path.set_file_name("pitch");
			fs::read_to_string(path).ok()
		})
		.and_then(|value| value.trim().parse().ok())
		.unwrap_or(47)
}

fn speak_as_letters(text: &str, letter_pitch_percent_change: u32, tts: &mut Tts) {
	let mut ssml = String::from("<speak>");
	let uppercase_pitch = 100u32.saturating_add(letter_pitch_percent_change);
	for ch in text.chars() {
		if ch.is_uppercase() {
			ssml.push_str(&format!("<prosody pitch=\"{uppercase_pitch}%\">{ch}</prosody>"));
		} else {
			ssml.push(ch);
		}
	}
	ssml.push_str("</speak>");
	if tts.speak_ssml(&ssml).is_err() {
		speak_in_chunks(text, tts);
	}
}

fn speak_in_chunks(text: &str, tts: &mut Tts) {
	let mut chunk = String::new();
	for word in text.split_whitespace() {
		if chunk.is_empty() {
			chunk.push_str(word);
			continue;
		}
		let next_len = chunk.len() + 1 + word.len();
		if next_len > MAX_LINE_LENGTH {
			speak(&chunk, tts);
			chunk.clear();
		} else {
			chunk.push(' ');
		}
		chunk.push_str(word);
	}
	if !chunk.is_empty() {
		speak(&chunk, tts);
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
