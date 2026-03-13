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
	reportOutputDir         = "gateway-api-conformance-reports"
	reportDatePlaceholder   = "-"
	switchboardImageToken   = "__SWITCHBOARD_IMAGE__"
)

// GatewayAPIConformanceSuite mirrors Traefik's conformance architecture:
// k3s bootstrap -> install manifests -> run official Gateway API conformance.
type GatewayAPIConformanceSuite struct {
	suite.Suite

	k3sContainer *k3s.K3sContainer
	kubeClient   client.Client
	restConfig   *rest.Config
	clientSet    *kclientset.Clientset
}

func TestGatewayAPIConformanceSuite(t *testing.T) {
	suite.Run(t, new(GatewayAPIConformanceSuite))
}

func (s *GatewayAPIConformanceSuite) SetupSuite() {
	ctx := s.T().Context()

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

	require.NoError(s.T(), s.k3sContainer.LoadImages(ctx, *switchboardImage))

	exitCode, _, err := s.k3sContainer.Exec(
		ctx,
		[]string{"kubectl", "wait", "-n", switchboardNamespace, switchboardDeployment, "--for=condition=Available", "--timeout=120s"},
	)
	require.Truef(s.T(), err == nil && exitCode == 0, "switchboard deployment did not become available: exit=%d err=%v", exitCode, err)

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
		return
	}

	data, err := io.ReadAll(output)
	if err != nil {
		s.T().Logf("failed to read switchboard logs output: %v", err)
		return
	}

	s.T().Logf("switchboard logs:\n%s", string(data))
}
