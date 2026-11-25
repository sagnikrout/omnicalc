#!/usr/bin/env bash
set -euo pipefail

if [ -z "${OMNICALC_BIN:-}" ]; then
    if command -v omnicalc >/dev/null 2>&1; then
        OMNICALC_BIN="$(command -v omnicalc)"
    elif [ -x "./target/release/omnicalc" ]; then
        OMNICALC_BIN="./target/release/omnicalc"
    elif [ -x "$HOME/bin/omnicalc" ]; then
        OMNICALC_BIN="$HOME/bin/omnicalc"
    else
        echo "Error: omnicalc executable not found on PATH or target/release/omnicalc" >&2
        exit 1
    fi
fi

TARGET_SHELL="${1:-$(basename "${SHELL:-bash}")}"

case "$TARGET_SHELL" in
    bash)
        DEST_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/bash-completion/completions"
        mkdir -p "$DEST_DIR"
        "$OMNICALC_BIN" --completions bash > "$DEST_DIR/omnicalc"
        echo "Installed bash completions to $DEST_DIR/omnicalc"
        ;;
    zsh)
        DEST_DIR="${ZDOTDIR:-$HOME}/.zfunc"
        mkdir -p "$DEST_DIR"
        "$OMNICALC_BIN" --completions zsh > "$DEST_DIR/_omnicalc"
        echo "Installed zsh completions to $DEST_DIR/_omnicalc (ensure fpath+=($DEST_DIR) in ~/.zshrc)"
        ;;
    fish)
        DEST_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions"
        mkdir -p "$DEST_DIR"
        "$OMNICALC_BIN" --completions fish > "$DEST_DIR/omnicalc.fish"
        echo "Installed fish completions to $DEST_DIR/omnicalc.fish"
        ;;
    *)
        echo "Error: unsupported shell '$TARGET_SHELL'. Use: bash, zsh, or fish." >&2
        exit 1
        ;;
esac
