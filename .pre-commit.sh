#!/bin/sh

set -eu

if ! cargo fmt -- --check; then
    echo "There are some code style issues."
    echo "Run cargo fmt first."
    exit 1
fi

if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "There are some clippy issues."
    exit 1
fi

if ! cargo check --all; then
    echo "There are some check issues."
    exit 1
fi

if ! cargo test --all-features; then
    echo "There are some test issues."
    exit 1
fi

exit 0
