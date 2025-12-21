#! /bin/bash
cargo build --release --bin sbk

cp target/release/sbk .
cd "$(dirname "$0")"
