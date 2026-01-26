<script lang="ts">
	import { Filter, Plus } from 'lucide-svelte';

	type Props = {
		filters: Record<string, any>;
		searchTerm: string;
		selectedId?: string;
		onSelect: (id: string) => void;
		onAdd: () => void;
	};

	let { filters, searchTerm, selectedId, onSelect, onAdd }: Props = $props();

	// 过滤后的列表
	let filteredList: [string, any][] = $derived.by(() => {
		if (!searchTerm.trim()) {
			return Object.entries(filters);
		}
		const search = searchTerm.toLowerCase();
		return Object.entries(filters).filter(
			([id, data]) =>
				id.toLowerCase().includes(search) || (data.class?.toLowerCase() || '').includes(search)
		);
	});

	let isEmpty = $derived(filteredList.length === 0);
</script>

<div class="filter-list">
	{#if isEmpty}
		<!-- 空状态：引导按钮 -->
		<div class="empty-state">
			{#if searchTerm.trim()}
				<div class="empty-icon">
					<Filter class="size-8" />
				</div>
				<p class="empty-text">No filters match your search</p>
				<button
					class="btn btn-sm preset-tonal"
					onclick={() => {
						searchTerm = '';
					}}
				>
					Clear search
				</button>
			{:else}
				<div class="empty-icon">
					<Filter class="size-8" />
				</div>
				<p class="empty-text">No filters created yet</p>
				<button class="btn btn-sm preset-filled-secondary" onclick={onAdd}>
					<Plus class="size-3" /> Create your first filter
				</button>
			{/if}
		</div>
	{:else}
		<!-- 过滤器列表 -->
		<div class="filter-items">
			{#each filteredList as [id, data]}
				<button
					type="button"
					class="filter-item"
					class:selected={selectedId === id}
					onclick={() => onSelect(id)}
					onkeydown={(e) => {
						if (e.key === 'Enter' || e.key === ' ') {
							e.preventDefault();
							onSelect(id);
						}
					}}
					aria-label={`Select filter ${id}`}
					tabindex="0"
				>
					<Filter class="item-icon" />
					<div class="item-content">
						<div class="item-name">{id}</div>
						<div class="item-class">{data.class || 'Not configured'}</div>
					</div>
					{#if data.class}
						<div class="status-dot configured" />
					{:else}
						<div class="status-dot unconfigured" />
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.filter-list {
		padding: 8px;
		flex: 1;
		overflow-y: auto;
	}

	.filter-items {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.filter-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 12px;
		width: 100%;
		border: none;
		background: transparent;
		border-radius: 6px;
		text-align: left;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.filter-item:hover {
		background: var(--surface-100-900);
	}

	.filter-item.selected {
		background: var(--primary-500/10);
	}

	.item-icon {
		size: 16px;
		color: var(--surface-500-400);
		flex-shrink: 0;
	}

	.item-content {
		flex: 1;
		min-width: 0;
		overflow: hidden;
	}

	.item-name {
		font-size: 13px;
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.item-class {
		font-size: 11px;
		color: var(--surface-400-600);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.status-dot.configured {
		background: var(--success-600-400);
	}

	.status-dot.unconfigured {
		background: var(--warning-600-400);
	}

	/* 空状态样式 */
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 32px 16px;
		text-align: center;
	}

	.empty-icon {
		width: 48px;
		height: 48px;
		border-radius: 50%;
		background: var(--surface-100-900);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--surface-400-600);
		margin-bottom: 16px;
	}

	.empty-text {
		font-size: 13px;
		color: var(--surface-500-400);
		margin-bottom: 16px;
	}
</style>
