use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;

use crate::clipboard;
use crate::context;
use crate::history::HistoryEntry;

pub struct PickerInput {
	pub items: Vec<DisplayEntry>,
	pub all_entries: Vec<HistoryEntry>,
	pub context_lines: usize,
	pub home_dir: Option<String>,
}

pub struct DisplayEntry {
	pub command: String,
	pub directory_display: String,
	pub timestamp_display: String,
	pub tail_index: usize,
}

#[derive(PartialEq)]
enum Mode {
	Select,
	Context,
}

fn set_raw_mode(fd: i32) -> Option<libc::termios> {
	unsafe {
		let mut original: libc::termios = std::mem::zeroed();
		if libc::tcgetattr(fd, &mut original) != 0 {
			return None;
		}
		let mut raw = original;
		raw.c_lflag &= !(libc::ICANON | libc::ECHO);
		raw.c_cc[libc::VMIN] = 1;
		raw.c_cc[libc::VTIME] = 0;
		if libc::tcsetattr(fd, libc::TCSANOW, &raw) != 0 {
			return None;
		}
		Some(original)
	}
}

fn restore_mode(fd: i32, original: &libc::termios) {
	unsafe {
		libc::tcsetattr(fd, libc::TCSANOW, original);
	}
}

pub fn run(input: &PickerInput) {
	if input.items.is_empty() {
		eprintln!("no results");
		return;
	}

	let count = input.items.len();
	render_menu(input);

	let tty = match std::fs::File::open("/dev/tty") {
		Ok(file) => file,
		Err(_) => return,
	};
	let fd = tty.as_raw_fd();
	let original_termios = match set_raw_mode(fd) {
		Some(termios) => termios,
		None => return,
	};

	let mut reader = io::BufReader::new(&tty);
	let mut mode = Mode::Select;
	let mut digit_buffer = String::new();

	loop {
		let mut byte = [0u8; 1];
		if reader.read(&mut byte).unwrap_or(0) == 0 {
			break;
		}
		let character = byte[0] as char;

		match character {
			'q' | 'Q' | '\x1b' => {
				if mode == Mode::Context {
					mode = Mode::Select;
					digit_buffer.clear();
					reprint_prompt(&mode);
					continue;
				}
				ewriteln("");
				break;
			}
			'c' | 'C' if mode == Mode::Select && digit_buffer.is_empty() => {
				mode = Mode::Context;
				digit_buffer.clear();
				reprint_prompt(&mode);
			}
			'\r' | '\n' => {
				ewriteln("");
				if digit_buffer.is_empty() {
					if mode == Mode::Context {
						mode = Mode::Select;
						reprint_prompt(&mode);
						continue;
					}
					break;
				}
				if let Ok(number) = digit_buffer.parse::<usize>() {
					if number >= 1 && number <= count {
						handle_selection(input, number - 1, &mode);
						if mode == Mode::Context {
							mode = Mode::Select;
							digit_buffer.clear();
							reprint_prompt(&mode);
							continue;
						}
						break;
					}
				}
				digit_buffer.clear();
			}
			'0'..='9' => {
				let mut candidate = digit_buffer.clone();
				candidate.push(character);
				let number = candidate.parse::<usize>().unwrap_or(0);

				if number < 1 || number > count {
					continue;
				}

				digit_buffer = candidate;
				ewrite(&character.to_string());

				if number * 10 > count {
					ewriteln("");
					handle_selection(input, number - 1, &mode);
					if mode == Mode::Context {
						mode = Mode::Select;
						digit_buffer.clear();
						reprint_prompt(&mode);
						continue;
					}
					break;
				}
			}
			'\x7f' | '\x08' => {
				if digit_buffer.pop().is_some() {
					ewrite("\x08 \x08");
				}
			}
			_ => {}
		}
	}

	restore_mode(fd, &original_termios);
}

fn render_menu(input: &PickerInput) {
	let mut stderr = io::stderr();
	let count = input.items.len();
	let number_width = if count >= 100 { 3 } else if count >= 10 { 2 } else { 1 };

	let max_directory_width = input
		.items
		.iter()
		.map(|item| item.directory_display.len())
		.max()
		.unwrap_or(0)
		.min(30);

	let terminal_width = std::env::var("COLUMNS")
		.ok()
		.and_then(|value| value.parse::<usize>().ok())
		.unwrap_or(80);

	let overhead = number_width + 2 + max_directory_width + 2 + 12 + 4;
	let command_width = terminal_width.saturating_sub(overhead).max(20);

	writeln!(stderr).ok();
	for (index, item) in input.items.iter().enumerate() {
		let command_display = truncate(&item.command, command_width);
		let directory_display = truncate(&item.directory_display, max_directory_width);
		writeln!(
			stderr,
			" {:>nw$}) {:<cw$}  {:<dw$}  {}",
			index + 1,
			command_display,
			directory_display,
			item.timestamp_display,
			nw = number_width,
			cw = command_width,
			dw = max_directory_width,
		).ok();
	}
	writeln!(stderr).ok();
	write!(stderr, " pick (c=context, q=quit): ").ok();
	stderr.flush().ok();
}

fn handle_selection(input: &PickerInput, index: usize, mode: &Mode) {
	let item = &input.items[index];
	match mode {
		Mode::Select => {
			if clipboard::yank(&item.command) {
				eprintln!(" yanked: {}", item.command);
			} else {
				eprintln!(" (no clipboard tool found)");
				eprintln!(" {}", item.command);
			}
		}
		Mode::Context => {
			context::show(
				&input.all_entries,
				item.tail_index,
				input.context_lines,
				&input.home_dir,
			);
		}
	}
}

fn reprint_prompt(mode: &Mode) {
	let label = match mode {
		Mode::Select => " pick (c=context, q=quit): ",
		Mode::Context => " context for (q=back): ",
	};
	let mut stderr = io::stderr();
	write!(stderr, "{}", label).ok();
	stderr.flush().ok();
}

fn truncate(text: &str, max_width: usize) -> String {
	if text.len() <= max_width {
		text.to_string()
	} else if max_width > 1 {
		format!("{}…", &text[..max_width - 1])
	} else {
		"…".to_string()
	}
}

fn ewrite(text: &str) {
	let mut stderr = io::stderr();
	write!(stderr, "{}", text).ok();
	stderr.flush().ok();
}

fn ewriteln(text: &str) {
	let mut stderr = io::stderr();
	writeln!(stderr, "{}", text).ok();
	stderr.flush().ok();
}
