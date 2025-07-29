<script lang="ts">
	import { onMount } from 'svelte';
	import FlowchartEditor from '$lib/components/sbh-flow/FlowchartEditor.svelte';
	import type { FlowchartState, NodeType } from '$lib/types/sbh-flow';

	let flowchartState: FlowchartState = $state({
		nodes: [],
		connections: [],
		selectedNodes: [],
		dragState: {
			isDragging: false
		},
		connectionState: {
			isConnecting: false
		},
		viewBox: {
			x: 0,
			y: 0,
			width: 1200,
			height: 800,
			scale: 1
		}
	});

	function handleStateChange(event: FlowchartState) {
		flowchartState = event;
	}

	function handleNodeConfigOpen(event: { nodeId: string }) {
		const { nodeId } = event;
		console.log('Open config for node:', nodeId);
		// 这里可以打开节点配置对话框
	}

	function handleSave(event: FlowchartState) {
		const state = event;
		console.log('Saving flowchart state:', state);
		// 这里可以保存到后端
		localStorage.setItem('flowchart-state', JSON.stringify(state));
	}

	function handleLoad() {
		const saved = localStorage.getItem('flowchart-state');
		if (saved) {
			try {
				flowchartState = JSON.parse(saved);
			} catch (e) {
				console.error('Failed to load saved state:', e);
			}
		}
	}

	onMount(() => {
		handleLoad();
	});
</script>

<svelte:head>
	<title>路由构建器 - Switchboard Admin</title>
</svelte:head>

<div class="h-full">
	<div class="mb-4">
		<h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">路由构建器</h1>
		<p class="text-gray-600 dark:text-gray-400">通过可视化流程图构建 HTTP 路由</p>
	</div>

	<div
		class="h-[calc(100vh-200px)] overflow-hidden rounded-lg border border-gray-200 dark:border-gray-700"
	>
		<FlowchartEditor
			state={flowchartState}
			onstatechange={handleStateChange}
			onnodeconfigopen={handleNodeConfigOpen}
			onsave={handleSave}
			onload={handleLoad}
		/>
	</div>
</div>
