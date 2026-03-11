export type K8sEnvResponse = {
	in_cluster: boolean;
	current_namespace: string | null;
};

export type K8sNamespacesResponse = {
	namespaces: string[];
};
