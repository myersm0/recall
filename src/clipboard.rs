use std::io::Write;
use std::process::{Command, Stdio};

pub enum YankResult {
	Success,
	Osc52,
	Failure(String),
}

pub fn yank(text: &str) -> YankResult {
	if cfg!(target_os = "macos") {
		if try_command("pbcopy", &[], text) {
			return YankResult::Success;
		}
	} else {
		let on_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
		let on_x11 = std::env::var("DISPLAY").is_ok();

		if on_wayland && try_command("wl-copy", &[], text) {
			return YankResult::Success;
		}
		if on_x11 && try_command("xclip", &["-selection", "clipboard"], text) {
			return YankResult::Success;
		}
		if on_x11 && try_command("xsel", &["--clipboard", "--input"], text) {
			return YankResult::Success;
		}

		if osc52(text) {
			return YankResult::Osc52;
		}

		let hint = if on_wayland {
			"install wl-copy (wl-clipboard package)"
		} else if on_x11 {
			"install xclip or xsel"
		} else {
			"no display server detected"
		};
		return YankResult::Failure(hint.to_string());
	}

	if osc52(text) {
		return YankResult::Osc52;
	}
	YankResult::Failure("no clipboard tool available".to_string())
}

fn try_command(program: &str, arguments: &[&str], text: &str) -> bool {
	let child = Command::new(program)
		.args(arguments)
		.stdin(Stdio::piped())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn();

	if let Ok(mut child) = child {
		if let Some(ref mut stdin) = child.stdin {
			if stdin.write_all(text.as_bytes()).is_ok() {
				drop(child.stdin.take());
				return child.wait().map(|s| s.success()).unwrap_or(false);
			}
		}
	}
	false
}

fn osc52(text: &str) -> bool {
	use base64::Engine;
	use base64::engine::general_purpose::STANDARD;

	let encoded = STANDARD.encode(text.as_bytes());
	if let Ok(mut tty) = std::fs::OpenOptions::new().write(true).open("/dev/tty") {
		let sequence = format!("\x1b]52;c;{}\x07", encoded);
		return tty.write_all(sequence.as_bytes()).is_ok();
	}
	false
}
