package k8sconformance

import (
	"flag"
)

var (
	switchboardImage                 = flag.String("switchboardImage", "switchboard/sbc:conformance", "switchboard image for conformance tests")
	sbkImage                         = flag.String("sbkImage", "switchboard/sbk:conformance", "sbk image for conformance tests")
	showLogs                         = flag.Bool("showLogs", false, "always show switchboard logs")
	gatewayAPIConformanceRunTest     = flag.String("gatewayAPIConformanceRunTest", "", "runs a specific Gateway API conformance test")
	switchboardImplementationVersion = flag.String("switchboardVersion", "local-dev", "switchboard implementation version for report")
)

const (
	k3sImage              = "docker.io/rancher/k3s:v1.34.2-k3s1"
	gatewayClassName      = "switchboard"
	switchboardNamespace  = "switchboard"
	switchboardDeployment = "deployments/switchboard"
	sbkDeployment         = "deployments/sbk"
)
