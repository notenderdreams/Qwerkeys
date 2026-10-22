#!/usr/bin/env bash
# Symlink custom git hooks from .hooks/ into .git/hooks/
#
# Usage: scripts/setup-hooks.sh

set -euo pipefail

cd "$(dirname "$0")/.."

HOOKS_DIR=".hooks"
GIT_HOOKS_DIR=".git/hooks"

if [[ ! -d ".git" ]]; then
  echo "error: .git directory not found" >&2
  exit 1
fi

mkdir -p "$GIT_HOOKS_DIR"

for hook in "$HOOKS_DIR"/*; do
  [[ -f "$hook" ]] || continue
  name="$(basename "$hook")"
  chmod +x "$hook"
  ln -sf "../../$hook" "$GIT_HOOKS_DIR/$name"
  echo "✓ Linked $hook -> $GIT_HOOKS_DIR/$name"
done

echo "✓ Git hooks installed successfully."
