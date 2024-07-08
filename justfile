alias b := build
alias c := check
alias t := test
alias d := develop
alias dc := develop-client

# COMMANDS -----------------------------------------------------------------------------------------

# List commands
default:
    @just --list

# Check
check:
    cargo check && cargo clippy --all -- -W clippy::all

# Test
test: check
    cargo test --all

# Build
build: test
    cargo build --release

# Recompile then restart the server whenever any change is made
develop:
    RUST_LOG="debug" cargo watch -q -c -w src/ -x "run"

# Re-run quick development queries whenever any change is made
develop-client:
    cargo watch -q -c -w tests/ -w src/ -x "test -q quick_dev -- --ignored --nocapture"
