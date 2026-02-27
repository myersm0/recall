use std::collections::HashSet;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

#[derive(Debug, Clone)]
pub struct HistoryEntry {
	pub timestamp: String,
	pub directory: String,
	pub command: String,
	pub tail_index: usize,
}

pub fn read_tail(path: &Path, max_lines: usize) -> Vec<HistoryEntry> {
	let lines = match tail_lines(path, max_lines) {
		Ok(lines) => lines,
		Err(_) => return Vec::new(),
	};
	lines
		.into_iter()
		.enumerate()
		.filter_map(|(index, line)| parse_entry(&line, index))
		.collect()
}

fn tail_lines(path: &Path, count: usize) -> std::io::Result<Vec<String>> {
	let mut file = std::fs::File::open(path)?;
	let file_size = file.metadata()?.len();

	if file_size == 0 {
		return Ok(Vec::new());
	}

	let chunk_size: u64 = 8192;
	let mut newline_positions: Vec<u64> = Vec::new();
	let mut position = file_size;

	loop {
		let read_start = position.saturating_sub(chunk_size);
		let read_length = (position - read_start) as usize;

		file.seek(SeekFrom::Start(read_start))?;
		let mut buffer = vec![0u8; read_length];
		file.read_exact(&mut buffer)?;

		for offset in (0..buffer.len()).rev() {
			if buffer[offset] == b'\n' {
				newline_positions.push(read_start + offset as u64);
			}
		}

		if newline_positions.len() > count + 1 || read_start == 0 {
			break;
		}
		position = read_start;
	}

	let start_byte = if newline_positions.len() > count {
		newline_positions[count] + 1
	} else {
		0
	};

	file.seek(SeekFrom::Start(start_byte))?;
	let mut tail_content = String::new();
	file.read_to_string(&mut tail_content)?;

	Ok(tail_content
		.lines()
		.filter(|line| !line.trim().is_empty())
		.map(String::from)
		.collect())
}

fn parse_entry(line: &str, tail_index: usize) -> Option<HistoryEntry> {
	let parts: Vec<&str> = line.splitn(3, ',').collect();
	if parts.len() != 3 {
		return None;
	}
	let timestamp = parts[0].to_string();
	let directory = decode_base64(parts[1])?;
	let command = decode_base64(parts[2])?;
	Some(HistoryEntry { timestamp, directory, command, tail_index })
}

fn decode_base64(encoded: &str) -> Option<String> {
	let bytes = STANDARD.decode(encoded.trim()).ok()?;
	String::from_utf8(bytes).ok()
}

pub fn dedup(entries: &[HistoryEntry]) -> Vec<HistoryEntry> {
	let mut seen = HashSet::new();
	entries
		.iter()
		.rev()
		.filter(|entry| seen.insert(entry.command.clone()))
		.cloned()
		.collect::<Vec<_>>()
		.into_iter()
		.rev()
		.collect()
}

pub fn filter(
	entries: Vec<HistoryEntry>,
	command_regex: &Option<regex::Regex>,
	directory_regex: &Option<regex::Regex>,
	prefix: bool,
) -> Vec<HistoryEntry> {
	let current_directory = if prefix {
		std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string())
	} else {
		None
	};

	entries
		.into_iter()
		.filter(|entry| {
			if let Some(ref pattern) = command_regex {
				if !pattern.is_match(&entry.command) {
					return false;
				}
			}
			if let Some(ref pattern) = directory_regex {
				if !pattern.is_match(&entry.directory) {
					return false;
				}
			}
			if let Some(ref current) = current_directory {
				if !entry.directory.starts_with(current.as_str()) {
					return false;
				}
			}
			true
		})
		.collect()
}
