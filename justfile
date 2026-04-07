set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]

default:
    @vx just --list

fmt:
    vx cargo fmt --all

fmt-check:
    vx cargo fmt --all -- --check

check:
    vx cargo check

lint:
    vx cargo clippy -- -D warnings

test:
    vx cargo test --locked

ci:
    vx just fmt-check
    vx just check
    vx just lint
    vx just test

build-release *args:
    vx cargo build --release --locked {{args}}

build-release-target target:
    vx cargo build --release --locked --target {{target}}

run *args:
    vx cargo run -- {{args}}

docs-install:
    cd docs && vx npm install

docs-dev:
    cd docs && vx npm run docs:dev

docs-build:
    cd docs && vx npm run docs:build

docs-preview:
    cd docs && vx npm run docs:preview
