//go:build gatewayAPIConformance

package k8sconformance

import (
	"context"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"slices"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	"github.com/testcontainers/testcontainers-go/modules/k3s"
	apiextensionsv1 "k8s.io/apiextensions-apiserver/pkg/apis/apiextensions/v1"
	"k8s.io/apimachinery/pkg/util/sets"
	kclientset "k8s.io/client-go/kubernetes"
	"k8s.io/client-go/rest"
	"k8s.io/client-go/tools/clientcmd"
	"sigs.k8s.io/controller-runtime/pkg/client"
	gatev1 "sigs.k8s.io/gateway-api/apis/v1"
	gatev1alpha2 "sigs.k8s.io/gateway-api/apis/v1alpha2"
	gatev1beta1 "sigs.k8s.io/gateway-api/apis/v1beta1"
	"sigs.k8s.io/gateway-api/conformance"
	v1 "sigs.k8s.io/gateway-api/conformance/apis/v1"
	"sigs.k8s.io/gateway-api/conformance/tests"
	"sigs.k8s.io/gateway-api/conformance/utils/config"
	ksuite "sigs.k8s.io/gateway-api/conformance/utils/suite"
	"sigs.k8s.io/yaml"
)

const (
	crdManifestPath         = "fixtures/gateway-api-conformance/00-experimental-v1.4.1.yml"
	rbacManifestPath        = "fixtures/gateway-api-conformance/01-rbac.yml"
	switchboardManifestTmpl = "fixtures/gateway-api-conformance/02-switchboard.yml.tmpl"
	controllerLogCollectCmd = "kubectl logs -n switchboard deploy/switchboard"
	sbkLogCollectCmd        = "kubectl logs -n switchboard deploy/sbk"
	resourceDumpCollectCmd  = "kubectl get gatewayclass,gateway,httproute -A -o yaml"
	eventsCollectCmd        = "kubectl get events -A --sort-by=.lastTimestamp"
	reportOutputDir         = "gateway-api-conformance-reports"
	logOutputDir            = "logs"
	reportDatePlaceholder   = "-"
	switchboardImageToken   = "__SWITCHBOARD_IMAGE__"
	sbkImageToken           = "__SBK_IMAGE__"
)

// GatewayAPIConformanceSuite mirrors Traefik's conformance architecture:
// k3s bootstrap -> install manifests -> run official Gateway API conformance.
type GatewayAPIConformanceSuite struct {
	suite.Suite

	k3sContainer *k3s.K3sContainer
	kubeClient   client.Client
	restConfig   *rest.Config
	clientSet    *kclientset.Clientset
	logDir       string
}

func TestGatewayAPIConformanceSuite(t *testing.T) {
	suite.Run(t, new(GatewayAPIConformanceSuite))
}

func (s *GatewayAPIConformanceSuite) SetupSuite() {
	ctx := s.T().Context()
	s.logDir = filepath.Join(logOutputDir, time.Now().UTC().Format("20060102T150405Z"))
	require.NoError(s.T(), os.MkdirAll(s.logDir, 0o755))

	manifestPath, err := s.renderSwitchboardManifest()
	require.NoError(s.T(), err)

	s.k3sContainer, err = k3s.Run(
		ctx,
		k3sImage,
		k3s.WithManifest(crdManifestPath),
		k3s.WithManifest(rbacManifestPath),
		k3s.WithManifest(manifestPath),
	)
	require.NoError(s.T(), err)

	require.NoError(s.T(), s.k3sContainer.LoadImages(ctx, *switchboardImage, *sbkImage))

	exitCode, _, err := s.k3sContainer.Exec(
		ctx,
		[]string{"kubectl", "wait", "-n", switchboardNamespace, switchboardDeployment, "--for=condition=Available", "--timeout=120s"},
	)
	require.Truef(s.T(), err == nil && exitCode == 0, "switchboard deployment did not become available: exit=%d err=%v", exitCode, err)

	exitCode, _, err = s.k3sContainer.Exec(
		ctx,
		[]string{"kubectl", "wait", "-n", switchboardNamespace, sbkDeployment, "--for=condition=Available", "--timeout=120s"},
	)
	require.Truef(s.T(), err == nil && exitCode == 0, "sbk deployment did not become available: exit=%d err=%v", exitCode, err)

	kubeConfigYaml, err := s.k3sContainer.GetKubeConfig(ctx)
	require.NoError(s.T(), err)

	s.restConfig, err = clientcmd.RESTConfigFromKubeConfig(kubeConfigYaml)
	require.NoError(s.T(), err)

	s.kubeClient, err = client.New(s.restConfig, client.Options{})
	require.NoError(s.T(), err)

	s.clientSet, err = kclientset.NewForConfig(s.restConfig)
	require.NoError(s.T(), err)

	require.NoError(s.T(), gatev1alpha2.Install(s.kubeClient.Scheme()))
	require.NoError(s.T(), gatev1beta1.Install(s.kubeClient.Scheme()))
	require.NoError(s.T(), gatev1.Install(s.kubeClient.Scheme()))
	require.NoError(s.T(), apiextensionsv1.AddToScheme(s.kubeClient.Scheme()))
}

func (s *GatewayAPIConformanceSuite) TearDownSuite() {
	if s.k3sContainer == nil {
		return
	}

	ctx := s.T().Context()
	if s.T().Failed() || *showLogs {
		s.dumpK3sLogs(ctx)
		s.dumpControllerLogs(ctx)
		s.dumpSbkLogs(ctx)
		s.dumpResourceSnapshot(ctx)
		s.dumpClusterEvents(ctx)
	}

	require.NoError(s.T(), s.k3sContainer.Terminate(ctx))
}

func (s *GatewayAPIConformanceSuite) TestGatewayAPIConformanceHTTPAndTLS() {
	cSuite, err := ksuite.NewConformanceTestSuite(ksuite.ConformanceOptions{
		Client:               s.kubeClient,
		Clientset:            s.clientSet,
		GatewayClassName:     gatewayClassName,
		Debug:                true,
		CleanupBaseResources: true,
		RestConfig:           s.restConfig,
		TimeoutConfig:        config.DefaultTimeoutConfig(),
		ManifestFS:           []fs.FS{&conformance.Manifests},
		RunTest:              *gatewayAPIConformanceRunTest,
		Implementation: v1.Implementation{
			Organization: "switchboard",
			Project:      "switchboard",
			URL:          "https://github.com/4t145/switchboard",
			Version:      *switchboardImplementationVersion,
			Contact:      []string{"@switchboard/maintainers"},
		},
		ConformanceProfiles: sets.New(
			ksuite.GatewayHTTPConformanceProfileName,
			ksuite.GatewayTLSConformanceProfileName,
		),
		EnableAllSupportedFeatures: true,
	})
	require.NoError(s.T(), err)

	cSuite.Setup(s.T(), tests.ConformanceTests)

	err = cSuite.Run(s.T(), tests.ConformanceTests)
	require.NoError(s.T(), err)

	report, err := cSuite.Report()
	require.NoError(s.T(), err)

	report.Date = reportDatePlaceholder
	slices.SortFunc(report.ProfileReports, func(a, b v1.ProfileReport) int {
		return strings.Compare(a.Name, b.Name)
	})

	reportBytes, err := yaml.Marshal(report)
	require.NoError(s.T(), err)

	versionDir := filepath.Join(reportOutputDir, report.GatewayAPIVersion)
	require.NoError(s.T(), os.MkdirAll(versionDir, 0o755))

	fileName := fmt.Sprintf("%s-%s-%s-report.yaml", report.GatewayAPIChannel, report.Version, report.Mode)
	outputPath := filepath.Join(versionDir, fileName)
	require.NoError(s.T(), os.WriteFile(outputPath, reportBytes, 0o600))
	s.T().Logf("Conformance report written to: %s", outputPath)
}

func (s *GatewayAPIConformanceSuite) renderSwitchboardManifest() (string, error) {
	tmpl, err := os.ReadFile(switchboardManifestTmpl)
	if err != nil {
		return "", err
	}

	rendered := strings.ReplaceAll(string(tmpl), switchboardImageToken, *switchboardImage)
	rendered = strings.ReplaceAll(rendered, sbkImageToken, *sbkImage)
	path := filepath.Join(s.T().TempDir(), "02-switchboard.yml")
	if err := os.WriteFile(path, []byte(rendered), 0o600); err != nil {
		return "", err
	}

	return path, nil
}

func (s *GatewayAPIConformanceSuite) dumpK3sLogs(ctx context.Context) {
	logs, err := s.k3sContainer.Logs(ctx)
	if err != nil {
		s.T().Logf("failed to read k3s logs: %v", err)
		return
	}

	data, err := io.ReadAll(logs)
	if err != nil {
		s.T().Logf("failed to consume k3s logs: %v", err)
		return
	}
	if err := s.writeLogArtifact("k3s.log", data); err != nil {
		s.T().Logf("failed to write k3s logs file: %v", err)
	}

	s.T().Logf("k3s logs:\n%s", string(data))
}

func (s *GatewayAPIConformanceSuite) dumpControllerLogs(ctx context.Context) {
	parts := []string{"sh", "-c", controllerLogCollectCmd}
	exitCode, output, err := s.k3sContainer.Exec(ctx, parts)
	if err != nil {
		s.T().Logf("failed to fetch switchboard logs: %v", err)
		return
	}
	if exitCode != 0 {
		s.T().Logf("switchboard logs command exited with %d", exitCode)
		if writeErr := s.writeLogArtifact("switchboard.log", []byte("<switchboard logs unavailable>\n")); writeErr != nil {
			s.T().Logf("failed to write switchboard logs placeholder file: %v", writeErr)
		}
		return
	}

	data, err := io.ReadAll(output)
	if err != nil {
		s.T().Logf("failed to read switchboard logs output: %v", err)
		return
	}
	if err := s.writeLogArtifact("switchboard.log", data); err != nil {
		s.T().Logf("failed to write switchboard logs file: %v", err)
	}

	s.T().Logf("switchboard logs:\n%s", string(data))
}

func (s *GatewayAPIConformanceSuite) dumpSbkLogs(ctx context.Context) {
	parts := []string{"sh", "-c", sbkLogCollectCmd}
	exitCode, output, err := s.k3sContainer.Exec(ctx, parts)
	if err != nil {
		s.T().Logf("failed to fetch sbk logs: %v", err)
		return
	}
	if exitCode != 0 {
		s.T().Logf("sbk logs command exited with %d", exitCode)
		if writeErr := s.writeLogArtifact("sbk.log", []byte("<sbk logs unavailable>\n")); writeErr != nil {
			s.T().Logf("failed to write sbk logs placeholder file: %v", writeErr)
		}
		return
	}

	data, err := io.ReadAll(output)
	if err != nil {
		s.T().Logf("failed to read sbk logs output: %v", err)
		return
	}
	if err := s.writeLogArtifact("sbk.log", data); err != nil {
		s.T().Logf("failed to write sbk logs file: %v", err)
	}

	s.T().Logf("sbk logs:\n%s", string(data))
}

func (s *GatewayAPIConformanceSuite) dumpResourceSnapshot(ctx context.Context) {
	s.dumpCommandOutputToFile(ctx, resourceDumpCollectCmd, "resources.yaml")
}

func (s *GatewayAPIConformanceSuite) dumpClusterEvents(ctx context.Context) {
	s.dumpCommandOutputToFile(ctx, eventsCollectCmd, "events.log")
}

func (s *GatewayAPIConformanceSuite) dumpCommandOutputToFile(ctx context.Context, cmd string, outputName string) {
	parts := []string{"sh", "-c", cmd}
	exitCode, output, err := s.k3sContainer.Exec(ctx, parts)
	if err != nil {
		s.T().Logf("failed to execute command %q: %v", cmd, err)
		return
	}
	if exitCode != 0 {
		s.T().Logf("command %q exited with %d", cmd, exitCode)
		return
	}

	data, err := io.ReadAll(output)
	if err != nil {
		s.T().Logf("failed to read command %q output: %v", cmd, err)
		return
	}

	if err := s.writeLogArtifact(outputName, data); err != nil {
		s.T().Logf("failed to write command %q output file: %v", cmd, err)
	}
}

func (s *GatewayAPIConformanceSuite) writeLogArtifact(name string, data []byte) error {
	outputPath := filepath.Join(s.logDir, name)
	if err := os.WriteFile(outputPath, data, 0o600); err != nil {
		return err
	}
	s.T().Logf("Conformance log written to: %s", outputPath)
	return nil
}
