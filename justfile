build-all:
    cargo build --bin sbk
    cargo build --bin sbc

dev-sbk:
    cargo run --bin sbk -- config examples/config/kernel.toml

dev-sbc:
    cargo run --bin sbc -- start examples/config/controller.toml
    