#!/usr/bin/env bash

set -eou pipefail

REPO="EwenSellitto/skills"
SKILL_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY_NAME="okf-cli"
BINARY_DST="$HOME/.local/bin/$BINARY_NAME"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin) OS="macOS" ;;
    Linux)  OS="Linux" ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64 | amd64)  ARCH="x86_64" ;;
    aarch64 | arm64) ARCH="aarch64" ;;
    *)
        echo "Unsupported arch: $ARCH"
        exit 1
        ;;
esac

SKILL_TARGETS=("$HOME/.agents/skills")
if [[ -d "$HOME/.config/opencode/skills" ]]; then
    SKILL_TARGETS=("$HOME/.config/opencode/skills")
fi
if [[ -d "$HOME/.claude" ]]; then
    SKILL_TARGETS+=("$HOME/.claude/skills")
fi

echo "Downloading $BINARY_NAME ($OS-$ARCH)..."
mkdir -p "$HOME/.local/bin"
url="https://github.com/$REPO/releases/latest/download/okf-cli-$OS-$ARCH"
if command -v curl &>/dev/null; then
    curl -fsSL "$url" -o "$BINARY_DST.tmp"
elif command -v wget &>/dev/null; then
    wget -q "$url" -O "$BINARY_DST.tmp"
else
    echo "Error: need curl or wget"
    exit 1
fi
chmod +x "$BINARY_DST.tmp"
# Swap in atomically so a failed download never leaves a broken binary.
mv -f "$BINARY_DST.tmp" "$BINARY_DST"
echo "  -> $BINARY_DST"

echo "Linking skills..."
for target in "${SKILL_TARGETS[@]}"; do
    mkdir -p "$target"
    for skill in okf okf-retreive rust-dev; do
        src="$SKILL_DIR/$skill"
        link="$target/$skill"
        if [[ "$(readlink "$link" || true)" == "$src" ]]; then
            echo "  = $link (already linked)"
        elif [[ -e "$link" && ! -L "$link" ]]; then
            echo "  ! $link exists and is not a symlink, skipping"
        else
            ln -sfn "$src" "$link"
            echo "  -> $link"
        fi
    done
done

shell_config=""
case "$SHELL" in
    */zsh) shell_config="$HOME/.zshrc" ;;
    */bash) shell_config="$HOME/.bashrc" ;;
    */fish) shell_config="$HOME/.config/fish/config.fish" ;;
esac

if ! echo "$PATH" | tr ':' '\n' | grep -qx "$HOME/.local/bin"; then
    if [[ -n "$shell_config" ]] && ! grep -q '.local/bin' "$shell_config" 2>/dev/null; then
        echo "$HOME/.local/bin is not in PATH. Adding to $shell_config..."
        case "$SHELL" in
            */fish)
                echo "fish_add_path $HOME/.local/bin" >> "$shell_config"
                ;;
            *)
                echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$shell_config"
                ;;
        esac
        echo "Reload your shell: source $shell_config"
    fi
else
    echo "$HOME/.local/bin is already in PATH."
fi

echo "Install complete."
