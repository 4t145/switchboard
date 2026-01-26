<script lang="ts">
	type Props = {
		entrypoint: string;
		nodeIds: string[];
		filterIds: string[];
		activeTab: 'flow' | 'filters';
		onAdd: (type: 'node' | 'filter') => void;
		onUpdateEntrypoint: (id: string) => void;
		onSearch: (term: string) => void;
	};

	let {
		entrypoint,
		nodeIds,
		filterIds,
		activeTab,
		onAdd,
		onUpdateEntrypoint,
		onSearch
	}: Props = $props();

	let searchTerm = $state('');

	function handleSearch(e: Event) {
		const target = e.currentTarget as HTMLInputElement;
		searchTerm = target.value;
		onSearch(target.value);
	}
</script>

<header class="tree-header">
	{#if activeTab === 'flow'}
		<!-- Flow Tree Tab Header -->
		<div class="header-section">
			<label class="label">
				<span class="label-text-xs">Entry Point</span>
				<select
					class="select select-sm"
					value={entrypoint}
					onchange={(e) => onUpdateEntrypoint(e.currentTarget.value)}
				>
					<option value="">-- None --</option>
					{#each nodeIds as id}
						<option value={id}>{id}</option>
					{/each}
				</select>
			</label>

			<div class="btn-group">
				<button
					class="btn btn-sm preset-filled-primary flex-1"
					onclick={() => onAdd('node')}
				>
					+ Node
				</button>
				<button
					class="btn btn-sm preset-filled-secondary flex-1"
					onclick={() => onAdd('filter')}
				>
					+ Filter
				</button>
			</div>
		</div>
	{:else}
		<!-- Filters Tab Header -->
		<div class="header-section">
			<div class="search-input">
				<input
					type="text"
					class="input input-sm"
					placeholder="Search filters..."
					value={searchTerm}
					oninput={handleSearch}
				/>
			</div>
		</div>
	{/if}
</header>

<style>
	.tree-header {
		padding: 12px;
		border-bottom: 1px solid var(--surface-200-800);
		background: var(--surface-50-950);
	}

	.header-section {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.btn-group {
		display: flex;
		gap: 8px;
	}

	.search-input {
		width: 100%;
	}
</style>
