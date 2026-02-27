mod clipboard;
mod config;
mod context;
mod history;
mod picker;

use clap::Parser;
use crate::config::AppConfig;
use crate::picker::{DisplayEntry, PickerInput};

#[derive(Parser)]
#[command(name = "recall", about = "Search and act on shell command history")]
struct Cli {
	/// Filter commands by regex pattern
	pattern: Option<String>,

	/// Filter commands by regex (alias for positional)
	#[arg(short = 'r', long)]
	regex: Option<String>,

	/// Restrict to commands run under the current directory tree
	#[arg(short, long)]
	prefix: bool,

	/// Filter by directory pattern
	#[arg(short = 'd', long)]
	directory: Option<String>,

	/// Max results to show
	#[arg(short = 'n', long, default_value_t = 15)]
	number: usize,

	/// How many recent history entries to consider
	#[arg(short = 'H', long)]
	history_depth: Option<usize>,

	/// Show all occurrences (don't deduplicate)
	#[arg(short = 'a', long)]
	all: bool,

	/// Surrounding commands to show in context mode
	#[arg(long)]
	context_lines: Option<usize>,
}

fn main() {
	let cli = Cli::parse();
	let config = AppConfig::load();

	let history_depth = cli.history_depth.unwrap_or(config.default_history_depth);
	let context_lines = cli.context_lines.unwrap_or(config.context_lines);

	let command_pattern = cli.pattern.or(cli.regex);
	let command_regex = command_pattern
		.as_ref()
		.and_then(|pattern| regex::Regex::new(pattern).ok());
	let directory_regex = cli.directory
		.as_ref()
		.and_then(|pattern| regex::Regex::new(pattern).ok());

	let all_entries = history::read_tail(&config.history_path, history_depth);
	let filtered = history::filter(all_entries.clone(), &command_regex, &directory_regex, cli.prefix);

	let display_entries = if cli.all {
		filtered
	} else {
		history::dedup(&filtered)
	};

	let limited: Vec<_> = display_entries
		.iter()
		.rev()
		.take(cli.number)
		.cloned()
		.collect::<Vec<_>>()
		.into_iter()
		.rev()
		.collect();

	let home_dir = dirs::home_dir().map(|path| path.to_string_lossy().to_string());

	let items: Vec<DisplayEntry> = limited
		.iter()
		.map(|entry| {
			let directory_display = abbreviate_home(&entry.directory, &home_dir);
			let timestamp_display = shorten_timestamp(&entry.timestamp);
			DisplayEntry {
				command: entry.command.clone(),
				directory_display,
				timestamp_display,
				tail_index: entry.tail_index,
			}
		})
		.collect();

	let picker_input = PickerInput {
		items,
		all_entries,
		context_lines,
		home_dir,
	};

	picker::run(&picker_input);
}

fn abbreviate_home(path: &str, home_dir: &Option<String>) -> String {
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

fn shorten_timestamp(timestamp: &str) -> String {
	if let Some((_date, time)) = timestamp.split_once(' ') {
		if let Some(short_time) = time.get(..5) {
			return short_time.to_string();
		}
	}
	timestamp.to_string()
}
