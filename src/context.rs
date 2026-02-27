use std::io::{self, Write};
use crate::history::HistoryEntry;

pub fn show(
	all_entries: &[HistoryEntry],
	target_tail_index: usize,
	context_lines: usize,
	home_dir: &Option<String>,
) {
	let position = all_entries
		.iter()
		.position(|entry| entry.tail_index == target_tail_index);

	let position = match position {
		Some(position) => position,
		None => return,
	};

	let start = position.saturating_sub(context_lines);
	let end = (position + context_lines + 1).min(all_entries.len());
	let window = &all_entries[start..end];

	let max_directory_width = window
		.iter()
		.map(|entry| abbreviate_directory(&entry.directory, home_dir).len())
		.max()
		.unwrap_or(0);

	let mut stderr = io::stderr();
	writeln!(stderr).ok();

	for entry in window {
		let marker = if entry.tail_index == target_tail_index { "▸" } else { " " };
		let directory = abbreviate_directory(&entry.directory, home_dir);
		writeln!(
			stderr,
			"  {} {}  {:width$}  {}",
			marker,
			entry.timestamp,
			directory,
			entry.command,
			width = max_directory_width,
		).ok();
	}

	writeln!(stderr).ok();
	stderr.flush().ok();
}

fn abbreviate_directory(path: &str, home_dir: &Option<String>) -> String {
	if let Some(ref home) = home_dir {
		if let Some(rest) = path.strip_prefix(home.as_str()) {
			if rest.is_empty() {
				return "~".to_string();
			}
			return format!("~{}", rest);
		}
	}
	path.to_string()
}
