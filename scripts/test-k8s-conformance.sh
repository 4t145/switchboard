#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CONFORMANCE_DIR="${REPO_ROOT}/tests/k8s-conformance"
REPORT_DIR="${CONFORMANCE_DIR}/gateway-api-conformance-reports"

DEFAULT_IMAGE="switchboard/sbc:conformance"
DEFAULT_SBK_IMAGE="switchboard/sbk:conformance"
DEFAULT_GO_TEST_TIMEOUT="30m"

IMAGE="${DEFAULT_IMAGE}"
SBK_IMAGE="${DEFAULT_SBK_IMAGE}"
VERSION="local-dev"
RUN_TEST=""
SHOW_LOGS="false"
SKIP_BUILD="false"
CONTAINERFILE="publish/container/sbc-web.containerfile"
SBK_CONTAINERFILE="publish/container/sbk.containerfile"
USE_RSPROXY="${USE_RSPROXY:-false}"
ARTIFACTS_DIR="${REPO_ROOT}/artifacts/k8s-conformance"
GO_TEST_TIMEOUT="${DEFAULT_GO_TEST_TIMEOUT}"

usage() {
    cat <<'EOF'
Usage: scripts/test-k8s-conformance.sh [options]

Options:
  --image <image>               Switchboard image for conformance tests
  --sbk-image <image>           SBK image for conformance tests
  --version <version>           Implementation version written to reports
  --run-test <test-name>        Run a single Gateway API conformance test
  --show-logs <true|false>      Always print controller logs
  --skip-build <true|false>     Skip Docker image build step
  --containerfile <path>        Containerfile path relative to repo root
  --sbk-containerfile <path>    SBK containerfile path relative to repo root
  --artifacts-dir <dir>         Output directory for logs and copied reports
  --go-test-timeout <duration>  Go test timeout (default: 30m)
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
        --sbk-image)
            SBK_IMAGE="$2"
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
        --containerfile)
            CONTAINERFILE="$2"
            shift 2
            ;;
        --sbk-containerfile)
            SBK_CONTAINERFILE="$2"
            shift 2
            ;;
        --artifacts-dir)
            ARTIFACTS_DIR="$2"
            shift 2
            ;;
        --go-test-timeout)
            GO_TEST_TIMEOUT="$2"
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

normalize_or_remap() {
    local expected_key="$1"
    local default_value="$2"
    local current_value="$3"

    if [[ "${current_value}" != *=* ]]; then
        printf '%s' "${current_value}"
        return
    fi

    local key="${current_value%%=*}"
    local value="${current_value#*=}"
    case "${key}" in
        image)
            IMAGE="${value}"
            ;;
        sbk_image)
            SBK_IMAGE="${value}"
            ;;
        version)
            VERSION="${value}"
            ;;
        run_test)
            RUN_TEST="${value}"
            ;;
        show_logs)
            SHOW_LOGS="${value}"
            ;;
        skip_build)
            SKIP_BUILD="${value}"
            ;;
        containerfile)
            CONTAINERFILE="${value}"
            ;;
        sbk_containerfile)
            SBK_CONTAINERFILE="${value}"
            ;;
        timeout|go_test_timeout)
            GO_TEST_TIMEOUT="${value}"
            ;;
    esac

    if [[ "${key}" == "${expected_key}" ]]; then
        printf '%s' "${value}"
    else
        printf '%s' "${default_value}"
    fi
}

IMAGE="$(normalize_or_remap image "${DEFAULT_IMAGE}" "${IMAGE}")"
SBK_IMAGE="$(normalize_or_remap sbk_image "${DEFAULT_SBK_IMAGE}" "${SBK_IMAGE}")"
VERSION="$(normalize_or_remap version "${VERSION}" "${VERSION}")"
RUN_TEST="$(normalize_or_remap run_test "${RUN_TEST}" "${RUN_TEST}")"
SHOW_LOGS="$(normalize_or_remap show_logs "${SHOW_LOGS}" "${SHOW_LOGS}")"
SKIP_BUILD="$(normalize_or_remap skip_build "${SKIP_BUILD}" "${SKIP_BUILD}")"
CONTAINERFILE="$(normalize_or_remap containerfile "${CONTAINERFILE}" "${CONTAINERFILE}")"
SBK_CONTAINERFILE="$(normalize_or_remap sbk_containerfile "${SBK_CONTAINERFILE}" "${SBK_CONTAINERFILE}")"
GO_TEST_TIMEOUT="$(normalize_or_remap go_test_timeout "${GO_TEST_TIMEOUT}" "${GO_TEST_TIMEOUT}")"

require_cmd docker
require_cmd go

docker info >/dev/null

mkdir -p "${ARTIFACTS_DIR}"
GO_TEST_LOG="${ARTIFACTS_DIR}/go-test.log"
METADATA_FILE="${ARTIFACTS_DIR}/metadata.txt"

if ! is_true "${SKIP_BUILD}"; then
    echo "[conformance] Building image: ${IMAGE}"
    echo "[conformance] Using containerfile: ${CONTAINERFILE}"
    docker build \
        --build-arg "USE_RSPROXY=${USE_RSPROXY}" \
        -f "${REPO_ROOT}/${CONTAINERFILE}" \
        -t "${IMAGE}" \
        "${REPO_ROOT}"

    echo "[conformance] Building SBK image: ${SBK_IMAGE}"
    echo "[conformance] Using SBK containerfile: ${SBK_CONTAINERFILE}"
    docker build \
        --build-arg "USE_RSPROXY=${USE_RSPROXY}" \
        -f "${REPO_ROOT}/${SBK_CONTAINERFILE}" \
        -t "${SBK_IMAGE}" \
        "${REPO_ROOT}"
else
    echo "[conformance] Skipping image build"
fi

if ! docker image inspect "${IMAGE}" >/dev/null 2>&1; then
    echo "error: image not found in local daemon: ${IMAGE}" >&2
    echo "hint: run with --skip-build false or verify image tag matches --image" >&2
    exit 1
fi

if ! docker image inspect "${SBK_IMAGE}" >/dev/null 2>&1; then
    echo "error: sbk image not found in local daemon: ${SBK_IMAGE}" >&2
    echo "hint: build/tag sbk image or pass --sbk-image with an existing local tag" >&2
    exit 1
fi

IMAGE_ID="$(docker image inspect --format '{{.Id}}' "${IMAGE}")"

{
    echo "image=${IMAGE}"
    echo "sbk_image=${SBK_IMAGE}"
    echo "image_id=${IMAGE_ID}"
    echo "version=${VERSION}"
    echo "run_test=${RUN_TEST}"
    echo "show_logs=${SHOW_LOGS}"
    echo "skip_build=${SKIP_BUILD}"
    echo "containerfile=${CONTAINERFILE}"
    echo "sbk_containerfile=${SBK_CONTAINERFILE}"
    echo "go_test_timeout=${GO_TEST_TIMEOUT}"
} > "${METADATA_FILE}"

echo "[conformance] Running Gateway API conformance tests"

GO_TEST_ARGS=(
    ./...
    -v
    -tags
    gatewayAPIConformance
    -run
    GatewayAPIConformanceSuite
    -timeout
    "${GO_TEST_TIMEOUT}"
    -switchboardImage
    "${IMAGE}"
    -sbkImage
    "${SBK_IMAGE}"
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
