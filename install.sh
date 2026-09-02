#!/usr/bin/env bash
set -euo pipefail

ACTION="install"
REPO_URL="https://github.com/xscriptor/gitnapse.git"
BIN_NAME="gitnapse"
USE_SUDO="auto"
CLEANUP="false"

usage() {
  cat <<'EOF'
GitNapse Remote Installer (Linux/macOS)

Usage:
  install.sh [options]

Options:
  --action <install|uninstall>   Action to perform (default: install)
  --repo-url <url>               Git repository URL (default: official repo)
  --no-sudo                      Never use sudo (Linux package operations)
  --sudo                         Force sudo for Linux package operations
  --cleanup                      Also remove local GitNapse config/cache on uninstall
  -h, --help                     Show this help
EOF
}

log() { printf "[gitnapse-install] %s\n" "$*"; }
err() { printf "[gitnapse-install] ERROR: %s\n" "$*" >&2; }

while [[ $# -gt 0 ]]; do
  case "$1" in
    --action)
      ACTION="${2:-}"
      shift 2
      ;;
    --repo-url)
      REPO_URL="${2:-}"
      shift 2
      ;;
    --no-sudo)
      USE_SUDO="false"
      shift
      ;;
    --sudo)
      USE_SUDO="true"
      shift
      ;;
    --cleanup)
      CLEANUP="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      err "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ "$ACTION" != "install" && "$ACTION" != "uninstall" ]]; then
  err "--action must be install or uninstall"
  exit 1
fi

SUDO_CMD=""
if [[ "$USE_SUDO" == "true" ]]; then
  SUDO_CMD="sudo"
elif [[ "$USE_SUDO" == "auto" && "${EUID:-$(id -u)}" -ne 0 ]]; then
  SUDO_CMD="sudo"
fi

have_cmd() { command -v "$1" >/dev/null 2>&1; }

detect_pkg_manager_linux() {
  if have_cmd apt-get; then echo "apt"; return; fi
  if have_cmd pacman; then echo "pacman"; return; fi
  if have_cmd dnf; then echo "dnf"; return; fi
  if have_cmd yum; then echo "yum"; return; fi
  if have_cmd zypper; then echo "zypper"; return; fi
  if have_cmd rpm; then echo "rpm"; return; fi
  echo "none"
}

install_deps_linux() {
  local pm
  pm="$(detect_pkg_manager_linux)"
  log "Detected Linux package manager: $pm"
  case "$pm" in
    apt)
      $SUDO_CMD apt-get update -y
      $SUDO_CMD apt-get install -y curl wget git ca-certificates build-essential pkg-config libssl-dev
      ;;
    pacman)
      $SUDO_CMD pacman -Sy --noconfirm curl wget git base-devel openssl pkgconf ca-certificates
      ;;
    dnf)
      $SUDO_CMD dnf install -y curl wget git gcc gcc-c++ make openssl-devel pkgconf-pkg-config ca-certificates
      ;;
    yum)
      $SUDO_CMD yum install -y curl wget git gcc gcc-c++ make openssl-devel pkgconfig ca-certificates
      ;;
    zypper)
      $SUDO_CMD zypper --non-interactive refresh
      $SUDO_CMD zypper --non-interactive install curl wget git gcc gcc-c++ make libopenssl-devel pkg-config ca-certificates
      ;;
    rpm)
      err "rpm exists but no high-level manager detected. Install dependencies manually."
      err "Required: curl/wget, git, compiler toolchain, OpenSSL dev headers, pkg-config."
      ;;
    *)
      err "Unsupported Linux distribution. Install dependencies manually and rerun."
      ;;
  esac
}

install_deps_macos() {
  log "Detected macOS."
  if ! xcode-select -p >/dev/null 2>&1; then
    log "Xcode Command Line Tools not found; requesting installation..."
    xcode-select --install || true
    log "If installation prompt appears, complete it and rerun if needed."
  fi

  if ! have_cmd brew; then
    log "Homebrew not found. Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    if [[ -x /opt/homebrew/bin/brew ]]; then
      eval "$(/opt/homebrew/bin/brew shellenv)"
    elif [[ -x /usr/local/bin/brew ]]; then
      eval "$(/usr/local/bin/brew shellenv)"
    fi
  fi

  if ! have_cmd brew; then
    err "Homebrew installation failed or brew not in PATH."
    exit 1
  fi

  brew update
  brew install git curl wget pkg-config openssl@3
}

ensure_rust() {
  if have_cmd cargo && have_cmd rustc; then
    log "Rust toolchain already installed."
    return
  fi
  log "Rust not found. Installing rustup toolchain..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1090
  source "${HOME}/.cargo/env"
  if ! have_cmd cargo; then
    err "cargo not found after rustup installation."
    exit 1
  fi
}

install_gitnapse() {
  ensure_rust
  log "Installing ${BIN_NAME} from ${REPO_URL} ..."
  cargo install --git "$REPO_URL" --locked "$BIN_NAME"
  log "Installed. Binary should be available at: ${HOME}/.cargo/bin/${BIN_NAME}"
  log "Run: ${BIN_NAME} --help"
}

uninstall_gitnapse() {
  if have_cmd cargo; then
    if cargo uninstall "$BIN_NAME" >/dev/null 2>&1; then
      log "Uninstalled ${BIN_NAME}."
    else
      log "${BIN_NAME} is not currently installed via cargo."
    fi
  else
    log "cargo not found; skipping cargo uninstall."
  fi

  if [[ "$CLEANUP" == "true" ]]; then
    rm -rf "${HOME}/.config/GitNapse" "${HOME}/.cache/GitNapse"
    log "Removed local config/cache directories."
  fi
}

install_prerequisites() {
  local os
  os="$(uname -s || true)"
  case "$os" in
    Linux*) install_deps_linux ;;
    Darwin*) install_deps_macos ;;
    *)
      err "Unsupported OS: ${os}. This script supports Linux and macOS."
      exit 1
      ;;
  esac
}

main() {
  if [[ "$ACTION" == "install" ]]; then
    install_prerequisites
    install_gitnapse
  else
    uninstall_gitnapse
  fi
}

main
