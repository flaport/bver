#!/bin/sh
# bver installer for macOS and Linux
# Usage: curl -LsSf https://github.com/flaport/bver/releases/latest/download/install.sh | sh

set -e

REPO="flaport/bver"
BINARY_NAME="bver"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
detect_target() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        linux)
            case "$ARCH" in
                x86_64)
                    # Prefer musl for better portability
                    echo "x86_64-unknown-linux-musl"
                    ;;
                aarch64|arm64)
                    echo "aarch64-unknown-linux-gnu"
                    ;;
                *)
                    echo "Unsupported architecture: $ARCH" >&2
                    exit 1
                    ;;
            esac
            ;;
        darwin)
            case "$ARCH" in
                x86_64)
                    echo "x86_64-apple-darwin"
                    ;;
                arm64|aarch64)
                    echo "aarch64-apple-darwin"
                    ;;
                *)
                    echo "Unsupported architecture: $ARCH" >&2
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo "Unsupported OS: $OS" >&2
            exit 1
            ;;
    esac
}

# Get latest release version
get_latest_version() {
    curl -sL "https://api.github.com/repos/$REPO/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

main() {
    TARGET=$(detect_target)
    VERSION=$(get_latest_version)

    if [ -z "$VERSION" ]; then
        echo "Error: Could not determine latest version" >&2
        exit 1
    fi

    echo "Installing bver $VERSION for $TARGET..."

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    # Download and extract
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/$BINARY_NAME-$TARGET.tar.gz"
    echo "Downloading from $DOWNLOAD_URL..."

    curl -LsSf "$DOWNLOAD_URL" | tar -xzf - -C "$INSTALL_DIR"

    # Make executable
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    echo ""
    echo "Successfully installed bver to $INSTALL_DIR/$BINARY_NAME"
    echo ""

    # Check if install dir is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*)
            echo "Run 'bver --help' to get started."
            ;;
        *)
            echo "Add $INSTALL_DIR to your PATH to use bver:"
            echo ""
            echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
            echo ""
            echo "Add this line to your ~/.bashrc, ~/.zshrc, or equivalent."
            ;;
    esac
}

main
