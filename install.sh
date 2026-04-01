#!/bin/sh
# Universal installer for terminal — GPU-rendered terminal emulator with GTD.
# Usage: curl -fsSL https://raw.githubusercontent.com/CySpiegel/terminal/main/install.sh | sh
set -eu

REPO="CySpiegel/terminal"
BINARY_NAME="terminal"
GITHUB_API="https://api.github.com/repos/${REPO}/releases/latest"

# Colors (if terminal supports them)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    CYAN='\033[0;36m'
    RESET='\033[0m'
else
    RED='' GREEN='' YELLOW='' CYAN='' RESET=''
fi

info()  { printf "${CYAN}info${RESET}  %s\n" "$1"; }
ok()    { printf "${GREEN}ok${RESET}    %s\n" "$1"; }
warn()  { printf "${YELLOW}warn${RESET}  %s\n" "$1"; }
err()   { printf "${RED}error${RESET} %s\n" "$1" >&2; exit 1; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Darwin) echo "apple-darwin" ;;
        Linux)  echo "unknown-linux-gnu" ;;
        *)      err "Unsupported OS: $(uname -s). Supported: macOS, Linux" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64" ;;
        arm64|aarch64)  echo "aarch64" ;;
        *)              err "Unsupported architecture: $(uname -m). Supported: x86_64, aarch64" ;;
    esac
}

# Find a download tool
find_downloader() {
    if command -v curl >/dev/null 2>&1; then
        echo "curl"
    elif command -v wget >/dev/null 2>&1; then
        echo "wget"
    else
        err "Neither curl nor wget found. Please install one."
    fi
}

# Download a URL to a file
download() {
    url="$1"
    dest="$2"
    case "$DOWNLOADER" in
        curl) curl -fsSL "$url" -o "$dest" ;;
        wget) wget -q "$url" -O "$dest" ;;
    esac
}

# Resolve version
resolve_version() {
    if [ -n "${TERMINAL_VERSION:-}" ]; then
        echo "$TERMINAL_VERSION"
        return
    fi

    info "Fetching latest release..."
    tmpfile=$(mktemp)
    download "$GITHUB_API" "$tmpfile" || err "Failed to fetch latest release info"
    version=$(grep '"tag_name"' "$tmpfile" | head -1 | sed 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
    rm -f "$tmpfile"

    if [ -z "$version" ]; then
        err "Could not determine latest version. Set TERMINAL_VERSION manually."
    fi
    echo "$version"
}

# Verify checksum
verify_checksum() {
    file="$1"
    checksums_file="$2"
    filename=$(basename "$file")

    expected=$(grep "$filename" "$checksums_file" | awk '{print $1}')
    if [ -z "$expected" ]; then
        warn "No checksum found for $filename — skipping verification"
        return
    fi

    if command -v sha256sum >/dev/null 2>&1; then
        actual=$(sha256sum "$file" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        actual=$(shasum -a 256 "$file" | awk '{print $1}')
    else
        warn "No sha256sum or shasum found — skipping checksum verification"
        return
    fi

    if [ "$expected" != "$actual" ]; then
        err "Checksum mismatch for $filename!\n  Expected: $expected\n  Actual:   $actual"
    fi
    ok "Checksum verified"
}

# Main
main() {
    info "Installing ${BINARY_NAME}..."

    DOWNLOADER=$(find_downloader)
    OS=$(detect_os)
    ARCH=$(detect_arch)
    VERSION=$(resolve_version)
    TARGET="${ARCH}-${OS}"

    info "Platform: ${TARGET}"
    info "Version:  ${VERSION}"

    # Determine install directory
    if [ -n "${TERMINAL_INSTALL_DIR:-}" ]; then
        INSTALL_DIR="$TERMINAL_INSTALL_DIR"
    elif [ "$(uname -s)" = "Darwin" ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="${HOME}/.local/bin"
    fi

    # Create temp directory with cleanup trap
    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    # Download binary and checksums
    ARCHIVE="${BINARY_NAME}-${VERSION}-${TARGET}.tar.gz"
    ARCHIVE_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"
    CHECKSUMS_URL="https://github.com/${REPO}/releases/download/${VERSION}/checksums-sha256.txt"

    info "Downloading ${ARCHIVE}..."
    download "$ARCHIVE_URL" "${TMPDIR}/${ARCHIVE}" || err "Failed to download ${ARCHIVE_URL}"

    info "Downloading checksums..."
    download "$CHECKSUMS_URL" "${TMPDIR}/checksums-sha256.txt" || warn "Could not download checksums"

    # Verify checksum
    if [ -f "${TMPDIR}/checksums-sha256.txt" ]; then
        verify_checksum "${TMPDIR}/${ARCHIVE}" "${TMPDIR}/checksums-sha256.txt"
    fi

    # Extract
    info "Extracting..."
    tar xzf "${TMPDIR}/${ARCHIVE}" -C "${TMPDIR}"

    # Install
    mkdir -p "$INSTALL_DIR"
    if [ -w "$INSTALL_DIR" ]; then
        install -m 755 "${TMPDIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    else
        info "Elevated permissions required for ${INSTALL_DIR}"
        sudo install -m 755 "${TMPDIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    fi

    ok "Installed ${BINARY_NAME} ${VERSION} to ${INSTALL_DIR}/${BINARY_NAME}"

    # Check PATH
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            warn "${INSTALL_DIR} is not in your PATH"
            echo ""
            echo "  Add it by running:"
            echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
            echo ""
            echo "  To make it permanent, add that line to your shell config:"
            echo "    ~/.bashrc, ~/.zshrc, or ~/.profile"
            ;;
    esac
}

main
