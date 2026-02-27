#!/bin/sh
set -e

REPO="myersm0/recall"
BIN_DIR="${HOME}/.local/bin"
SHARE_DIR="${HOME}/.local/share/recall"

info() { printf "\033[0;34m%s\033[0m\n" "$*"; }
err()  { printf "\033[0;31m%s\033[0m\n" "$*" >&2; exit 1; }

detect_platform() {
	os="$(uname -s)"
	arch="$(uname -m)"

	case "$os" in
		Linux)  os="linux" ;;
		Darwin) os="macos" ;;
		*)      err "Unsupported OS: $os" ;;
	esac

	case "$arch" in
		x86_64|amd64)  arch="x86_64" ;;
		arm64|aarch64) arch="aarch64" ;;
		*)             err "Unsupported architecture: $arch" ;;
	esac

	echo "recall-${os}-${arch}"
}

main() {
	artifact="$(detect_platform)"
	info "Detected platform: ${artifact}"

	url="https://github.com/${REPO}/releases/latest/download/${artifact}.tar.gz"
	info "Downloading ${url}..."

	tmpdir="$(mktemp -d)"
	trap 'rm -rf "$tmpdir"' EXIT

	if command -v curl >/dev/null 2>&1; then
		curl -fsSL "$url" -o "${tmpdir}/recall.tar.gz"
	elif command -v wget >/dev/null 2>&1; then
		wget -qO "${tmpdir}/recall.tar.gz" "$url"
	else
		err "Neither curl nor wget found."
	fi

	tar xzf "${tmpdir}/recall.tar.gz" -C "$tmpdir"

	mkdir -p "$BIN_DIR" "$SHARE_DIR"
	cp "${tmpdir}/${artifact}" "${BIN_DIR}/recall"
	chmod +x "${BIN_DIR}/recall"
	info "Installed binary to ${BIN_DIR}/recall"

	if [ -f "${tmpdir}/shell/recall.sh" ]; then
		cp "${tmpdir}/shell/recall.sh" "${SHARE_DIR}/recall.sh"
	elif [ -f "${tmpdir}/recall.sh" ]; then
		cp "${tmpdir}/recall.sh" "${SHARE_DIR}/recall.sh"
	fi
	info "Installed shell functions to ${SHARE_DIR}/recall.sh"

	echo ""
	info "Add the following to your shell profile (.bashrc, .zshrc, etc.):"
	echo ""
	case ":$PATH:" in
		*":${BIN_DIR}:"*) ;;
		*) echo "  export PATH=\"\$HOME/.local/bin:\$PATH\"" ;;
	esac
	echo "  source \"\$HOME/.local/share/recall/recall.sh\""
	echo ""
	info "Then restart your shell."
}

main
