import type { K8sEnvResponse, K8sNamespacesResponse } from '../types';
import { fetchQuery } from './index';

export const k8sApi = {
	getEnv(): Promise<K8sEnvResponse> {
		return fetchQuery<K8sEnvResponse>('/api/k8s/env');
	},
	getNamespaces(): Promise<K8sNamespacesResponse> {
		return fetchQuery<K8sNamespacesResponse>('/api/k8s/namespaces');
	}
};
