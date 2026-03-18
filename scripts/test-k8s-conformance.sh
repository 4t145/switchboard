#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CONFORMANCE_DIR="${REPO_ROOT}/tests/k8s-conformance"
REPORT_DIR="${CONFORMANCE_DIR}/gateway-api-conformance-reports"

IMAGE="switchboard/all-in-one:conformance"
VERSION="local-dev"
RUN_TEST=""
SHOW_LOGS="false"
SKIP_BUILD="false"
USE_RSPROXY="${USE_RSPROXY:-false}"
ARTIFACTS_DIR="${REPO_ROOT}/artifacts/k8s-conformance"

usage() {
    cat <<'EOF'
Usage: scripts/test-k8s-conformance.sh [options]

Options:
  --image <image>               Switchboard image for conformance tests
  --version <version>           Implementation version written to reports
  --run-test <test-name>        Run a single Gateway API conformance test
  --show-logs <true|false>      Always print controller logs
  --skip-build <true|false>     Skip Docker image build step
  --artifacts-dir <dir>         Output directory for logs and copied reports
  --use-rsproxy <true|false>    Forward USE_RSPROXY build arg to Docker
  -h, --help                    Show this help
EOF
}

is_true() {
    local value="${1:-}"
    [[ "${value}" == "true" || "${value}" == "1" || "${value}" == "yes" ]]
}

require_cmd() {
    local cmd="$1"
    if ! command -v "${cmd}" >/dev/null 2>&1; then
        echo "error: required command not found: ${cmd}" >&2
        exit 1
    fi
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --image)
            IMAGE="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --run-test)
            if [[ $# -ge 2 && "${2}" != --* ]]; then
                RUN_TEST="$2"
                shift 2
            else
                RUN_TEST=""
                shift 1
            fi
            ;;
        --show-logs)
            SHOW_LOGS="$2"
            shift 2
            ;;
        --skip-build)
            SKIP_BUILD="$2"
            shift 2
            ;;
        --artifacts-dir)
            ARTIFACTS_DIR="$2"
            shift 2
            ;;
        --use-rsproxy)
            USE_RSPROXY="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "error: unknown argument: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

require_cmd docker
require_cmd go

docker info >/dev/null

mkdir -p "${ARTIFACTS_DIR}"
GO_TEST_LOG="${ARTIFACTS_DIR}/go-test.log"
METADATA_FILE="${ARTIFACTS_DIR}/metadata.txt"

if ! is_true "${SKIP_BUILD}"; then
    echo "[conformance] Building image: ${IMAGE}"
    docker build \
        --build-arg "USE_RSPROXY=${USE_RSPROXY}" \
        -f "${REPO_ROOT}/publish/container/switchboard-all-in-one.containerfile" \
        -t "${IMAGE}" \
        "${REPO_ROOT}"
else
    echo "[conformance] Skipping image build"
fi

IMAGE_ID="$(docker image inspect --format '{{.Id}}' "${IMAGE}")"

{
    echo "image=${IMAGE}"
    echo "image_id=${IMAGE_ID}"
    echo "version=${VERSION}"
    echo "run_test=${RUN_TEST}"
    echo "show_logs=${SHOW_LOGS}"
    echo "skip_build=${SKIP_BUILD}"
} > "${METADATA_FILE}"

echo "[conformance] Running Gateway API conformance tests"

GO_TEST_ARGS=(
    ./...
    -v
    -tags
    gatewayAPIConformance
    -run
    GatewayAPIConformanceSuite
    -switchboardImage
    "${IMAGE}"
    -switchboardVersion
    "${VERSION}"
)

if [[ -n "${RUN_TEST}" ]]; then
    GO_TEST_ARGS+=(
        -gatewayAPIConformanceRunTest
        "${RUN_TEST}"
    )
fi

if is_true "${SHOW_LOGS}"; then
    GO_TEST_ARGS+=(
        -showLogs
    )
fi

set +e
(
    cd "${CONFORMANCE_DIR}"
    go test "${GO_TEST_ARGS[@]}"
) 2>&1 | tee "${GO_TEST_LOG}"
TEST_EXIT_CODE=${PIPESTATUS[0]}
set -e

if [[ -d "${REPORT_DIR}" ]]; then
    REPORTS_ARTIFACT_DIR="${ARTIFACTS_DIR}/reports"
    rm -rf "${REPORTS_ARTIFACT_DIR}"
    mkdir -p "${REPORTS_ARTIFACT_DIR}"
    cp -R "${REPORT_DIR}"/. "${REPORTS_ARTIFACT_DIR}/"
fi

echo "[conformance] Test exit code: ${TEST_EXIT_CODE}"
echo "[conformance] Artifacts: ${ARTIFACTS_DIR}"

exit "${TEST_EXIT_CODE}"
