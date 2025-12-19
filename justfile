build-all:
    cargo build --bin sbk
    cargo build --bin sbc

dev-sbk:
    cargo run --bin sbk examples/config/kernel.toml
dev-sbk-sudo:
    cargo build --bin sbk
    sudo ./target/debug/sbk examples/config/kernel.toml
dev-sbc:
    cargo run --bin sbc -- start examples/config/controller.toml
    