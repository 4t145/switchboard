import { api } from '$lib/api/routes';

type K8sCapabilityState = {
	loading: boolean;
	loaded: boolean;
	available: boolean;
	currentNamespace: string | null;
	error: string | null;
};

class CapabilitiesStore {
	k8s = $state<K8sCapabilityState>({
		loading: false,
		loaded: false,
		available: false,
		currentNamespace: null,
		error: null
	});

	private pendingLoad: Promise<void> | null = null;

	async loadK8sEnv(force = false): Promise<void> {
		if (this.pendingLoad && !force) {
			return this.pendingLoad;
		}

		if (this.k8s.loading && !force) {
			return;
		}

		const task = this.runLoadK8sEnv();
		this.pendingLoad = task;

		try {
			await task;
		} finally {
			if (this.pendingLoad === task) {
				this.pendingLoad = null;
			}
		}
	}

	private async runLoadK8sEnv(): Promise<void> {
		this.k8s = {
			...this.k8s,
			loading: true,
			error: null
		};

		try {
			const env = await api.k8s.getEnv();
			this.k8s = {
				loading: false,
				loaded: true,
				available: env.in_cluster,
				currentNamespace: env.current_namespace,
				error: null
			};
		} catch (error) {
			this.k8s = {
				...this.k8s,
				loading: false,
				loaded: true,
				available: false,
				error: error instanceof Error ? error.message : 'Failed to load kubernetes environment.'
			};
		}
	}
}

export const capabilitiesStore = new CapabilitiesStore();
