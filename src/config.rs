use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppConfig {
	pub history_path: PathBuf,
	pub context_lines: usize,
	pub default_history_depth: usize,
}

impl Default for AppConfig {
	fn default() -> Self {
		let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
		Self {
			history_path: home.join(".bash_history_extended"),
			context_lines: 5,
			default_history_depth: 1000,
		}
	}
}

impl AppConfig {
	pub fn config_path() -> PathBuf {
		dirs::config_dir()
			.unwrap_or_else(|| PathBuf::from("."))
			.join("recall")
			.join("config.toml")
	}

	pub fn load() -> Self {
		let path = Self::config_path();
		if let Ok(contents) = std::fs::read_to_string(&path) {
			toml::from_str(&contents).unwrap_or_default()
		} else {
			Self::default()
		}
	}
}
