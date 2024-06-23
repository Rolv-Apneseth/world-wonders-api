alias b := build
alias t := test
alias ds := develop-server
alias dc := develop-client

# COMMANDS -----------------------------------------------------------------------------------------

# List commands
default:
    @just --list

# Build
build:
    cargo build --release

# Test
test:
    cargo test --all

# Recompile then run again whenever any change is made
develop-server:
    cargo watch -q -c -w src/ -x "run"

develop-client:
    cargo watch -q -c -w tests/ -w src/ -x "test -q quick_dev -- --nocapture"
