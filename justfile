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

test-k8s-conformance-all-in-one image="switchboard/all-in-one:conformance" sbk_image="switchboard/sbk:conformance" version="local-dev" run_test="" show_logs="true" skip_build="false" go_test_timeout="30m":
    bash scripts/test-k8s-conformance.sh \
        --image {{image}} \
        --sbk-image {{sbk_image}} \
        --version {{version}} \
        --run-test "{{run_test}}" \
        --show-logs {{show_logs}} \
        --skip-build {{skip_build}} \
        --go-test-timeout {{go_test_timeout}} \
        --containerfile publish/container/switchboard-all-in-one.containerfile \
        --sbk-containerfile publish/container/sbk.containerfile
        
test-k8s-conformance-sbc image="switchboard/sbc:conformance" sbk_image="switchboard/sbk:conformance" version="local-dev" run_test="" show_logs="true" skip_build="false" go_test_timeout="30m":
    bash scripts/test-k8s-conformance.sh \
        --image {{image}} \
        --sbk-image {{sbk_image}} \
        --version {{version}} \
        --run-test "{{run_test}}" \
        --show-logs {{show_logs}} \
        --skip-build {{skip_build}} \
        --go-test-timeout {{go_test_timeout}} \
        --containerfile publish/container/sbc-web.containerfile \
        --sbk-containerfile publish/container/sbk.containerfile
        
test-start-kind:
    bash tests/k8s/setup-kind.sh

test-k8s-conformance-local:
    cd tests/k8s-conformance && go test ./... -v -tags gatewayAPIConformance -run GatewayAPIConformanceSuite

debug-surreal:
    surreal start rocksdb://tmp/data/controller_storage.dbcd

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
