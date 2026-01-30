<script lang="ts">
	import { api } from '$lib/api/routes';
	import { FileText, CloudDownload, PlusSquare, Loader2 } from '@lucide/svelte';
	import ObjectPages from '$lib/components/object-pages.svelte';
	import { shortRev } from '$lib/utils';

	let {
		mode = $bindable('select'),
		canProceed = $bindable(false),
		isLoading = $bindable(false),
		onNext,
		initialResolver = 'fs',
		initialFsPath = ''
	} = $props<{
		mode?: 'select' | 'load-saved' | 'from-source';
		canProceed?: boolean;
		isLoading?: boolean;
		onNext: (config: Record<string, any>, summary: string, saveAs?: string) => void;
		initialResolver?: 'k8s' | 'fs';
		initialFsPath?: string;
	}>();

	// --- Load Saved State ---
	let selectedConfigId = $state<string | null>(null);
	let selectedItemDescriptor = $state<any>(null); // To store full descriptor for loading

	// --- From Source State ---
	let resolver = $state(initialResolver);
	let fsPath = $state(initialFsPath);
	let saveAs = $state<string>(''); // For from-source save_as parameter

	// Update canProceed based on current mode and form state
	$effect(() => {
		if (mode === 'select') {
			canProceed = false;
		} else if (mode === 'load-saved') {
			canProceed = !!selectedConfigId;
		} else if (mode === 'from-source') {
			canProceed = resolver === 'fs' ? !!fsPath : true;
		}
	});

	// --- Actions ---

	function handleCreateNew() {
		// Initialize an empty structure for ServiceConfig
		onNext(
			{
				tcp_services: [],
				tls: []
			},
			'New Empty Configuration'
		);
	}

	function startLoadSaved() {
		mode = 'load-saved';
		// Reset selection
		selectedConfigId = null;
		selectedItemDescriptor = null;
	}

	async function confirmLoadSaved() {
		if (!selectedItemDescriptor) return;

		isLoading = true;
		try {
			const response = await api.storage.get(selectedItemDescriptor);
			onNext(
				response as Record<string, any>,
				`Loaded: ${selectedItemDescriptor.id} (rev ${shortRev(selectedItemDescriptor.revision)})`
			);
		} catch (e) {
			console.error('Failed to fetch config details', e);
			alert('Failed to load configuration: ' + e);
		} finally {
			isLoading = false;
		}
	}

	function startFromSource() {
		mode = 'from-source';
	}

	async function confirmFromSource() {
		isLoading = true;
		try {
			const req = {
				resolver,
				config: resolver === 'fs' ? { path: fsPath } : {},
				save_as: saveAs || undefined
			};

			const response = await api.resolve.service_config(req);
			const summary = resolver === 'fs' ? `Source: File (${fsPath})` : `Source: Kubernetes`;
			onNext(response.config, summary, saveAs || undefined);
		} catch (e) {
			console.error('Failed to resolve config', e);
			alert('Failed to resolve configuration: ' + e);
		} finally {
			isLoading = false;
		}
	}

	// Expose functions for parent to call
	export { confirmLoadSaved, confirmFromSource };

	function goBack() {
		mode = 'select';
		selectedConfigId = null;
	}
</script>

{#if mode === 'select'}
	<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
		<!-- Create New -->
		<button
			class="flex cursor-pointer flex-col items-center space-y-4 card border border-surface-200 p-6 text-center card-hover transition-colors hover:bg-surface-200 dark:border-surface-700 dark:hover:bg-surface-800"
			onclick={handleCreateNew}
		>
			<div
				class="rounded-full bg-primary-100 p-4 text-primary-600 dark:bg-primary-900 dark:text-primary-300"
			>
				<PlusSquare size={48} />
			</div>
			<div>
				<h3 class="h3 font-bold">Create New</h3>
				<p class="mt-2 opacity-75">Start with a fresh, empty configuration.</p>
			</div>
		</button>

		<!-- Load Saved -->
		<button
			class="flex cursor-pointer flex-col items-center space-y-4 card border border-surface-200 p-6 text-center card-hover transition-colors hover:bg-surface-200 dark:border-surface-700 dark:hover:bg-surface-800"
			onclick={startLoadSaved}
		>
			<div
				class="rounded-full bg-secondary-100 p-4 text-secondary-600 dark:bg-secondary-900 dark:text-secondary-300"
			>
				<FileText size={48} />
			</div>
			<div>
				<h3 class="h3 font-bold">Load Saved</h3>
				<p class="mt-2 opacity-75">Load a previously saved configuration file.</p>
			</div>
		</button>

		<!-- From Source -->
		<button
			class="flex cursor-pointer flex-col items-center space-y-4 card border border-surface-200 p-6 text-center card-hover transition-colors hover:bg-surface-200 dark:border-surface-700 dark:hover:bg-surface-800"
			onclick={startFromSource}
		>
			<div
				class="rounded-full bg-tertiary-100 p-4 text-tertiary-600 dark:bg-tertiary-900 dark:text-tertiary-300"
			>
				<CloudDownload size={48} />
			</div>
			<div>
				<h3 class="h3 font-bold">From Source</h3>
				<p class="mt-2 opacity-75">Import configuration from an external source.</p>
			</div>
		</button>
	</div>
{:else if mode === 'load-saved'}
	<div class="animate-fade-in space-y-4">
		<h4 class="h4">选择已保存的配置</h4>

		<!-- Reusing ObjectPages Component -->
		<div
			class="h-[500px] overflow-y-auto rounded-lg border border-surface-200 p-2 dark:border-surface-700"
		>
			<ObjectPages
				pageSize={12}
				filter={{ data_type: 'ServiceConfig', latest_only: true, lockedFields: ['dataType'] }}
				selectionMode="single"
				selectedId={selectedConfigId}
				onSelect={(item) => {
					selectedConfigId = item.descriptor.id;
					selectedItemDescriptor = item.descriptor;
				}}
				showViewDetails={false}
				showEdit={false}
				showDelete={false}
			/>
		</div>
	</div>
{:else if mode === 'from-source'}
	<div class="animate-fade-in space-y-4">
		<h4 class="h4">配置源</h4>

		<label class="label">
			<span>Resolver Type</span>
			<select class="select" bind:value={resolver}>
				<option value="fs">Filesystem (Server Side)</option>
				<option value="k8s">Kubernetes Cluster</option>
			</select>
		</label>

		{#if resolver === 'fs'}
			<label class="label">
				<span>Config File Path</span>
				<input class="input" type="text" bind:value={fsPath} placeholder="/path/to/config.yaml" />
				<p class="text-sm opacity-75">
					Absolute path on the server where the controller is running.
				</p>
			</label>
		{:else}
			<div class="alert preset-tonal-info">
				Will attempt to load configuration from the Kubernetes cluster currently configured in the
				controller's environment.
			</div>
		{/if}

		<label class="label">
			<span>Save As (Optional)</span>
			<input class="input" type="text" bind:value={saveAs} placeholder="e.g. my-imported-config" />
			<p class="text-sm opacity-75">
				If provided, the resolved configuration will be saved to storage with this ID.
			</p>
		</label>
	</div>
{/if}
