build-all:
    cargo build --bin sbk
    cargo build --bin sbc
example-gen-tls:
    bash examples/config/tls/gen.sh
dev-sbk:
    cargo run --bin sbk examples/config/kernel.toml
dev-sbk-sudo:
    cargo build --bin sbk
    sudo ./target/debug/sbk examples/config/kernel.toml
dev-sbc:
    cargo run --bin sbc -- start examples/config/controller.toml
dev-sbc-sudo:
    cargo build --bin sbc
    sudo ./target/debug/sbc start examples/config/controller.toml