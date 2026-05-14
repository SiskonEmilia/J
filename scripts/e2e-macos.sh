#!/usr/bin/env bash
set -euo pipefail

EXE="$(cd "$(dirname "$0")/.." && pwd)/target/release/j"

if [ ! -x "$EXE" ]; then
    echo "ERROR: $EXE not found or not executable"
    exit 1
fi

WORKSPACE="$(mktemp -d)"
cleanup() { rm -rf "$WORKSPACE"; }
trap cleanup EXIT

export J_CONFIG="$WORKSPACE/config.jsonc"

TARGET_DIR="/tmp/j_e2e_target"
rm -rf "$TARGET_DIR"
mkdir -p "$TARGET_DIR"

echo "[INFO] Adding test project..."
"$EXE" :add testproj "$TARGET_DIR"

TEMP_HOME="$WORKSPACE/home"
mkdir -p "$TEMP_HOME"

TEST_SCRIPT="$WORKSPACE/test.zsh"
cat > "$TEST_SCRIPT" << 'ZSH_EOF'
j() {
  if [ "$#" -eq 0 ]; then
    "$J_EXE" :help
    return $?
  fi
  _j_out=$("$J_EXE" --shell=zsh "$@")
  _j_rc=$?
  if [ $_j_rc -ne 0 ]; then
    return $_j_rc
  fi
  if [ -z "$_j_out" ]; then
    return 0
  fi
  case "$_j_out" in
    cd\ --*)
      eval "$_j_out"
      ;;
    *)
      printf '%s\n' "$_j_out"
      ;;
  esac
}

# Test: jump with root name changes PWD
j testproj
if [ "$PWD" = "$TARGET_DIR" ]; then
  echo "[OK] j testproj changed \$PWD to $TARGET_DIR"
else
  echo "[FAIL] expected \$PWD=$TARGET_DIR got $PWD"
  exit 1
fi
ZSH_EOF

echo "[INFO] Running smoke test..."
HOME="$TEMP_HOME" J_EXE="$EXE" J_CONFIG="$J_CONFIG" \
  TARGET_DIR="$TARGET_DIR" /bin/zsh -f "$TEST_SCRIPT"

echo "ALL PASSED."
