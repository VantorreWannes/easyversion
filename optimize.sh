#!/bin/bash
set -euo pipefail

readonly TARGET="$(rustc -vV | sed -n 's/host: //p')"

readonly WORKLOAD_COUNT="${FILES_COUNT:-25}"
readonly WORKLOAD_TEMP_DIR="${TEMP_DIR:-"./target/$TARGET/files"}"

readonly BIN_PATH="$(pwd)/target/$TARGET/release/ev-bolt-instrumented"

readonly SPLIT_DIR="$(pwd)/target/$TARGET/split-$$"

export RUSTFLAGS="${RUSTFLAGS:--C target-feature=+aes,+sse2}"

cleanup() {
    if [ -d "$WORKLOAD_TEMP_DIR" ]; then
        cd "$WORKLOAD_TEMP_DIR" 2>/dev/null || return
        "$BIN_PATH" clean 2>/dev/null || true
        cd - >/dev/null 2>&1 || true
    fi
    if [ -d "$SPLIT_DIR" ]; then
        cd "$SPLIT_DIR" 2>/dev/null || true
        "$BIN_PATH" clean 2>/dev/null || true
        cd - >/dev/null 2>&1 || true
        rm -rf "$SPLIT_DIR"
    fi
}

create_workload() {
    mkdir -p "$WORKLOAD_TEMP_DIR"
    pushd "$WORKLOAD_TEMP_DIR" >/dev/null
    for i in $(seq 0 $((WORKLOAD_COUNT))); do
        dd if=/dev/urandom of="$i.bin" bs=1M count=$((i * 10)) status=none
    done
    popd >/dev/null
}

run_workload() {
    local bin="../release/ev-bolt-instrumented"

    pushd "$WORKLOAD_TEMP_DIR" >/dev/null

    "$bin" list
    "$bin" save -c "Initial Commit"

    "$bin" save -c "Added backup"
    "$bin" list

    "$bin" save -c "Removed large file"
    "$bin" list

    trap cleanup EXIT

    "$bin" split -p "$SPLIT_DIR" -o
    "$bin" split -p "$SPLIT_DIR" -o

    popd >/dev/null
}

main() {
    echo ":: Building for Target: $TARGET"

    cargo pgo test
    cargo pgo bench

    cargo pgo bolt build --with-pgo -- --package cli --target "$TARGET"

    run_workload

    cargo pgo bolt optimize --with-pgo -- --package cli --target "$TARGET"

    echo ":: Done: ./target/$TARGET/release/ev-bolt-optimized"
}

main "$@"
