label := "dev"
proxy := "false"

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
    cargo run --bin sbc -- start --config examples/config/controller.toml --no-web-root
dev-sbc-sudo:
    cargo build --bin sbc
    sudo ./target/debug/sbc start examples/config/controller.toml

build-container:
    podman build \
        --build-arg USE_RSPROXY={{proxy}} \
        --network host \
        -f publish/container/sbk.containerfile \
        -t switchboard/sbk:{{label}} .

test-start-kind:
    bash tests/k8s/setup-kind.sh

test-k8s-conformance-local:
    cd tests/k8s-conformance && go test ./... -v -tags gatewayAPIConformance -run GatewayAPIConformanceSuite

debug-surreal:
    surreal start rocksdb://tmp/data/controller_storage.db

install-uipro:
    npm install -g uipro-cli

install-agent-browser:
    npm install -g agent-browser

install-skill-ui-ux-pro-max:
    uipro install --ai opencode

install-skill-agent-browser:
    mkdir -p .opencode/skills/agent-browser
    curl -o .opencode/skills/agent-browser/SKILL.md \
        https://raw.githubusercontent.com/vercel-labs/agent-browser/main/skills/agent-browser/SKILL.md
