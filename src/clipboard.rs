use std::io::Write;
use std::process::{Command, Stdio};

pub fn yank(text: &str) -> bool {
	let candidates = if cfg!(target_os = "macos") {
		vec!["pbcopy"]
	} else {
		vec!["xclip -selection clipboard", "xsel --clipboard --input"]
	};

	for candidate in candidates {
		let parts: Vec<&str> = candidate.split_whitespace().collect();
		let program = parts[0];
		let arguments = &parts[1..];

		if let Ok(mut child) = Command::new(program)
			.args(arguments)
			.stdin(Stdio::piped())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
		{
			if let Some(ref mut stdin) = child.stdin {
				if stdin.write_all(text.as_bytes()).is_ok() {
					drop(child.stdin.take());
					return child.wait().map(|s| s.success()).unwrap_or(false);
				}
			}
		}
	}
	false
}
