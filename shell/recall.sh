#!/usr/bin/env bash

# History logging hook
_recall_last_logged_line=0
_recall_log_command() {
	local current_line
	current_line=$(history 1 | awk '{print $1}')
	[[ "$current_line" == "$_recall_last_logged_line" ]] && return
	_recall_last_logged_line="$current_line"

	local timestamp
	timestamp=$(date '+%Y-%m-%d %H:%M:%S')
	local current_dir="$PWD"
	local cmd
	cmd=$(history 1 | sed 's/^[ ]*[0-9]\+[ ]*//')

	local dir_encoded cmd_encoded
	dir_encoded=$(printf '%s' "$current_dir" | base64 | tr -d '\n')
	cmd_encoded=$(printf '%s' "$cmd" | base64 | tr -d '\n')
	echo "$timestamp,$dir_encoded,$cmd_encoded" >> "$HOME/.bash_history_extended"
}

if [[ -n "$PROMPT_COMMAND" ]]; then
	PROMPT_COMMAND="_recall_log_command; $PROMPT_COMMAND"
else
	PROMPT_COMMAND="_recall_log_command"
fi

recall() {
	command recall "$@"
}
