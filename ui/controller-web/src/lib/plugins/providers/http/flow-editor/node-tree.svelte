<script lang="ts">
	import TreeHeader from './tree-header.svelte';
	import TreeNode from './tree-node.svelte';
	import FilterList from './filter-list.svelte';
	import { buildFlowTree, buildFlowTreeWithResolve } from './tree-builder';
	import type { TreeNode as TreeNodeType, TreeConnection } from './tree-builder';

	type Props = {
		nodes: Record<string, any>;
		filters: Record<string, any>;
		entrypoint: string;
		selectedId?: string;
		selectedType?: 'node' | 'filter';
		onSelect: (id: string, type: 'node' | 'filter') => void;
		onAdd: (type: 'node' | 'filter') => void;
		onUpdateEntrypoint: (entrypoint: string) => void;
		onUpdateNodes?: (nodes: Record<string, any>) => void;
		onUpdateFilters?: (filters: Record<string, any>) => void;
	};

	let {
		nodes,
		filters,
		entrypoint,
		selectedId,
		selectedType,
		onSelect,
		onAdd,
		onUpdateEntrypoint,
		onUpdateNodes,
		onUpdateFilters
	}: Props = $props();

	// 状态
	let activeTab = $state<'flow' | 'filters'>('flow');
	let expandedNodes = $state<Set<string>>(new Set());
	let searchTerm = $state('');

	// 批量解析状态
	let resolveState = $state<{
		isResolving: boolean;
		total: number;
		completed: number;
		failed: number;
	}>({
		isResolving: false,
		total: 0,
		completed: 0,
		failed: 0
	});

	// 计算属性 - 使用同步版本（解析完成后会更新）
	let logicalTree = $derived(buildFlowTree(nodes, filters, entrypoint));

	// 批量解析逻辑函数（在 effect 外部，避免状态ful mutation）
	async function processBatchResolution() {
		if (!nodes || Object.keys(nodes).length === 0 || activeTab !== 'flow') {
			return;
		}

		resolveState = {
			isResolving: true,
			total: 0,
			completed: 0,
			failed: 0
		};

		try {
			const result = await buildFlowTreeWithResolve(
				nodes,
				filters,
				entrypoint,
				(progress) => {
					resolveState = progress;
				}
			);

			if (result.updatedNodes) {
				onUpdateNodes?.(result.updatedNodes);
			}

			logicalTree = buildFlowTree(result.updatedNodes || nodes, result.updatedFilters || filters, entrypoint);

			resolveState = {
				isResolving: false,
				total: result.resolveState.total,
				completed: result.resolveState.completed,
				failed: result.resolveState.failed
			};
		} catch (e) {
			console.error('Failed to resolve links and build tree:', e);
			resolveState = {
				isResolving: false,
				total: 0,
				completed: 0,
				failed: 1
			};
		}
	}

	function handleToggleExpand(nodeId: string) {
		const newSet = new Set(expandedNodes);
		if (newSet.has(nodeId)) {
			newSet.delete(nodeId);
		} else {
			newSet.add(nodeId);
		}
		expandedNodes = newSet;
	}

	function handleSelect(id: string, type: 'node' | 'filter') {
		onSelect(id, type);
	}

	
	// 自动触发批量链接解析（移除 nodes, activeTab 依赖，使用函数封装避免状态ful mutation）
	$effect(() => {
		processBatchResolution();
	});
</script>

<div class="node-tree">
	<!-- 顶部 Tab -->
	<div class="tree-tabs">
		<button
				class="tab-item"
				class:active={activeTab === 'flow'}
				onclick={() => (activeTab = 'flow')}
		>
			Flow Tree
		</button>
		<button
				class="tab-item"
				class:active={activeTab === 'filters'}
				onclick={() => (activeTab = 'filters')}
		>
			Filters ({Object.keys(filters).length})
		</button>
	</div>

	<!-- 批量解析进度条 -->
	{#if resolveState.isResolving}
		<div class="resolve-progress">
			<div class="progress-header">
				<span class="progress-icon">⚡</span>
				<span class="progress-text">
					Resolving {resolveState.completed} / {resolveState.total} links...
				</span>
			</div>
			{#if resolveState.total > 0}
				<div class="progress-bar-container">
					<div
						class="progress-bar"
						style:width="{(resolveState.completed / resolveState.total) * 100}%"
					></div>
				</div>
			{/if}
		</div>
	{/if}

	<!-- 内容区域 -->
	{#if activeTab === 'flow'}
		<div class="tab-content">
			<TreeHeader
				entrypoint={entrypoint}
				nodeIds={Object.keys(nodes)}
				filterIds={Object.keys(filters)}
				activeTab={activeTab}
				onAdd={onAdd}
				onUpdateEntrypoint={onUpdateEntrypoint}
				onSearch={(term) => (searchTerm = term)}
			/>

			<div class="tree-container">
				{#if logicalTree.tree}
					<TreeNode
						node={logicalTree.tree!}
						depth={0}
						isExpanded={expandedNodes.has(logicalTree.tree!.id)}
						hasChildren={
							(logicalTree.tree!.children?.length ?? 0) > 0
						}
						onToggleExpand={() => handleToggleExpand(logicalTree.tree!.id)}
						onSelect={handleSelect}
						isSelected={selectedId === logicalTree.tree!.id && selectedType === 'node'}
					>
						<!-- 渲染子节点 -->
						{#if expandedNodes.has(logicalTree.tree.id)}
							{#each logicalTree.tree.children || [] as connection}
								<div class="connection-group">
									<!-- 连接节点 -->
									<TreeNode
										node={connection}
										depth={1}
										isExpanded={false}
										hasChildren={false}
										onToggleExpand={() => {}}
										onSelect={() => {}}
										isSelected={false}
									/>

									<!-- 连接的子节点 -->
									<div class="connection-children">
										{#if connection.filterNodes}
											{#each connection.filterNodes as filter}
												<TreeNode
													node={filter}
													depth={1}
													isExpanded={expandedNodes.has(filter.id)}
													hasChildren={false}
													onToggleExpand={() => handleToggleExpand(filter.id)}
													onSelect={handleSelect}
													isSelected={selectedId === filter.id && selectedType === 'filter'}
												/>
											{/each}
										{/if}

										{#if connection.targetNode}
											<TreeNode
												node={connection.targetNode}
												depth={1}
												isExpanded={expandedNodes.has(connection.targetNode?.id ?? '')}
												hasChildren={
													(connection.targetNode?.children?.length ?? 0) > 0
												}
												onToggleExpand={() => connection.targetNode && handleToggleExpand(connection.targetNode.id)}
												onSelect={handleSelect}
												isSelected={
													selectedId === connection.targetNode?.id && selectedType === 'node'
												}
											>
												<!-- 递归渲染目标节点的子节点 -->
												{#if expandedNodes.has(connection.targetNode.id)}
													{#each connection.targetNode.children || [] as childConnection}
														<div class="connection-group">
															<TreeNode
																node={childConnection}
																depth={2}
																isExpanded={false}
																hasChildren={false}
																onToggleExpand={() => {}}
																onSelect={() => {}}
																isSelected={false}
															/>

															<div class="connection-children">
																{#if childConnection.filterNodes}
																	{#each childConnection.filterNodes as filter}
																		<TreeNode
																			node={filter}
																			depth={2}
																			isExpanded={expandedNodes.has(filter.id)}
																			hasChildren={false}
																			onToggleExpand={() => handleToggleExpand(filter.id)}
																			onSelect={handleSelect}
																			isSelected={
																				selectedId === filter.id &&
																				selectedType === 'filter'
																			}
																		/>
																	{/each}
																{/if}

																{#if childConnection.targetNode}
																	<TreeNode
																		node={childConnection.targetNode}
																		depth={2}
																		isExpanded={expandedNodes.has(childConnection.targetNode?.id ?? '')}
																		hasChildren={
																			(childConnection.targetNode?.children?.length ?? 0) > 0
																		}
																		onToggleExpand={() => childConnection.targetNode && handleToggleExpand(childConnection.targetNode.id)}
																		onSelect={handleSelect}
																		isSelected={
																			selectedId === childConnection.targetNode?.id && selectedType === 'node'
																		}
																	>
																		{#if expandedNodes.has(
																			childConnection.targetNode.id
																		)}
																			{#each childConnection.targetNode.children ||
																			[] as nestedConnection}
																				<div class="connection-group">
																					<TreeNode
																						node={nestedConnection}
																						depth={3}
																						isExpanded={false}
																						hasChildren={false}
																						onToggleExpand={() => {}}
																						onSelect={() => {}}
																						isSelected={false}
																					/>

																					<div class="connection-children">
																						{#if nestedConnection.filterNodes}
																							{#each nestedConnection.filterNodes as filter}
																								<TreeNode
																									node={filter}
																									depth={3}
																									isExpanded={expandedNodes.has(filter.id)}
																									hasChildren={false}
																									onToggleExpand={() => handleToggleExpand(filter.id)}
																									onSelect={handleSelect}
																									isSelected={
																										selectedId === filter.id &&
																										selectedType === 'filter'
																									}
																								/>
																							{/each}
																						{/if}

																						{#if nestedConnection.targetNode}
															<TreeNode
																node={nestedConnection.targetNode}
																depth={3}
																isExpanded={expandedNodes.has(nestedConnection.targetNode?.id ?? '')}
																hasChildren={
																	(nestedConnection.targetNode?.children?.length ?? 0) > 0
																}
																onToggleExpand={() => nestedConnection.targetNode && handleToggleExpand(nestedConnection.targetNode.id)}
																onSelect={handleSelect}
																isSelected={
																	selectedId === nestedConnection.targetNode?.id &&
																	selectedType === 'node'
																}
															/>
																						{/if}
																					</div>
																				</div>
																			{/each}
																		{/if}
																	</TreeNode>
																{/if}
															</div>
														</div>
													{/each}
												{/if}
											</TreeNode>
										{/if}
									</div>
								</div>
							{/each}
						{/if}
					</TreeNode>
				{/if}

				<!-- 孤儿节点 -->
				{#if logicalTree.orphans.length > 0}
					<div class="orphans-section">
						<div class="orphans-header">Disconnected</div>
						{#each logicalTree.orphans as orphan}
							<TreeNode
								node={orphan}
								depth={0}
								isExpanded={expandedNodes.has(orphan.id)}
								hasChildren={orphan.children && orphan.children.length > 0}
								onToggleExpand={() => handleToggleExpand(orphan.id)}
								onSelect={handleSelect}
								isSelected={selectedId === orphan.id && selectedType === 'node'}
							/>
						{/each}
					</div>
				{/if}

				<!-- 空状态 -->
				{#if !logicalTree.tree && Object.keys(nodes).length === 0}
					<div class="empty-state">
						<p class="empty-text">No nodes created yet</p>
						<p class="empty-hint">Add a node to get started</p>
					</div>
				{/if}

				{#if !logicalTree.tree && Object.keys(nodes).length > 0 && !entrypoint}
					<div class="empty-state">
						<p class="empty-text">Select an entry point to view the flow tree</p>
					</div>
				{/if}
			</div>
		</div>
	{:else}
		<div class="tab-content">
			<TreeHeader
				entrypoint={entrypoint}
				nodeIds={Object.keys(nodes)}
				filterIds={Object.keys(filters)}
				activeTab={activeTab}
				onAdd={onAdd}
				onUpdateEntrypoint={onUpdateEntrypoint}
				onSearch={(term) => (searchTerm = term)}
			/>

			<FilterList
				filters={filters}
				searchTerm={searchTerm}
				selectedId={selectedId}
				onSelect={(id) => handleSelect(id, 'filter')}
				onAdd={() => onAdd('filter')}
			/>
		</div>
	{/if}
</div>

<style>
	.node-tree {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.tree-tabs {
		display: flex;
		border-bottom: 1px solid var(--surface-200-800);
		background: var(--surface-100-900);
	}

	.tab-item {
		flex: 1;
		padding: 12px 16px;
		font-size: 13px;
		font-weight: 500;
		border: none;
		background: transparent;
		cursor: pointer;
		color: var(--surface-500-400);
		transition: all 0.15s ease;
	}

	.tab-item:hover {
		color: var(--surface-900-50);
	}

	.tab-item.active {
		color: var(--primary-600-400);
		border-bottom: 2px solid var(--primary-600-400);
	}

	.tab-content {
		display: flex;
		flex-direction: column;
		flex: 1;
		overflow: hidden;
	}

	.tree-container {
		flex: 1;
		overflow-y: auto;
		padding: 8px;
	}

	.connection-group {
		position: relative;
	}

	.connection-children {
		padding-top: 4px;
		padding-bottom: 4px;
	}

	.orphans-section {
		margin-top: 16px;
		padding-top: 12px;
		border-top: 1px solid var(--surface-200-800);
	}

	.orphans-header {
		padding: 4px 12px;
		margin-bottom: 8px;
		font-size: 10px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--surface-400-600);
	}

	/* 批量解析进度条样式 */
	.resolve-progress {
		padding: 12px 16px;
		background: var(--surface-100-900);
		border-bottom: 1px solid var(--surface-200-800);
	}

	.progress-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 8px;
		font-size: 13px;
		font-weight: 500;
		color: var(--surface-700-300);
	}

	.progress-icon {
		font-size: 14px;
	}

	.progress-bar-container {
		height: 4px;
		background: var(--surface-200-800);
		border-radius: 2px;
		overflow: hidden;
	}

	.progress-bar {
		height: 100%;
		background: var(--primary-600-400);
		border-radius: 2px;
		transition: width 0.3s ease;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 32px 16px;
		text-align: center;
	}

	.empty-text {
		font-size: 13px;
		color: var(--surface-500-400);
		margin-bottom: 4px;
	}

	.empty-hint {
		font-size: 11px;
		color: var(--surface-400-600);
	}
</style>
