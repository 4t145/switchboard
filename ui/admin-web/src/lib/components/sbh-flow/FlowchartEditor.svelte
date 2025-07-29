<script lang="ts">
	import { Button, DropdownMenu } from 'bits-ui';
	import { Plus, Save, Download, Upload, ZoomIn, ZoomOut, RotateCcw } from 'lucide-svelte';

	import FlowNode from './FlowNode.svelte';
	import Connection from './Connection.svelte';
	import NodeToolbar from './NodeToolbar.svelte';

	import type {
		FlowchartState,
		FlowNode as FlowNodeType,
		Connection as ConnectionType,
		Position,
		NodeType
	} from '$lib/types/sbh-flow';

	interface Props {
		state: FlowchartState;
		onstatechange?: (state: FlowchartState) => void;
		onnodeconfigopen?: (data: { nodeId: string }) => void;
		onsave?: (state: FlowchartState) => void;
		onload?: () => void;
	}

	let { state, onstatechange, onnodeconfigopen, onsave, onload }: Props = $props();

	let svgElement: SVGSVGElement;
	let containerElement: HTMLDivElement;

	// Helper functions
	function generateId(): string {
		return Math.random().toString(36).substr(2, 9);
	}

	function createNode(type: NodeType, position: Position): FlowNodeType {
		const id = generateId();
		const node: FlowNodeType = {
			id,
			type,
			position,
			size: { width: 200, height: 80 },
			label: `${type} ${id.slice(0, 4)}`,
			selected: false, // 确保添加 selected 属性
			ports: {
				inputs:
					type !== 'service'
						? [
								{
									id: `${id}-input`,
									type: 'input',
									position: { x: 0, y: 40 },
									connected: false
								}
							]
						: [],
				outputs: [
					{
						id: `${id}-output`,
						type: 'output',
						position: { x: 200, y: 40 },
						connected: false
					}
				]
			}
		};
		console.log('Created node:', node); // 调试信息
		return node;
	}

	function getMousePosition(event: MouseEvent): Position {
		const rect = svgElement.getBoundingClientRect();
		// 获取相对于SVG元素的鼠标位置
		const clientX = event.clientX - rect.left;
		const clientY = event.clientY - rect.top;

		// 转换到SVG坐标系
		// 考虑viewBox的偏移和缩放
		const x = (clientX / rect.width) * state.viewBox.width + state.viewBox.x;
		const y = (clientY / rect.height) * state.viewBox.height + state.viewBox.y;
		return {
			x,
			y
		};
	}

	// Event handlers
	function handleAddNode(type: NodeType) {
		// 计算在当前视图中心的实际坐标
		const centerX = (-state.viewBox.x + state.viewBox.width / 2) / state.viewBox.scale;
		const centerY = (-state.viewBox.y + state.viewBox.height / 2) / state.viewBox.scale;

		const newNode = createNode(type, {
			x: centerX - 100, // 减去节点宽度的一半
			y: centerY - 40 // 减去节点高度的一半
		});

		const newState = {
			...state,
			nodes: [...state.nodes, newNode]
		};

		console.log('New state nodes:', newState.nodes); // 调试信息
		onstatechange?.(newState);
	}

	function handleNodeSelect(event: CustomEvent) {
		const { nodeId, multiSelect } = event.detail;

		let selectedNodes: string[];
		if (multiSelect) {
			selectedNodes = state.selectedNodes.includes(nodeId)
				? state.selectedNodes.filter((id) => id !== nodeId)
				: [...state.selectedNodes, nodeId];
		} else {
			selectedNodes = [nodeId];
		}

		const newState = {
			...state,
			selectedNodes,
			nodes: state.nodes.map((node) => ({
				...node,
				selected: selectedNodes.includes(node.id)
			}))
		};

		onstatechange?.(newState);
	}

	function handleNodeDrag(event: CustomEvent) {
		const { nodeId, position } = event.detail;

		const newState = {
			...state,
			nodes: state.nodes.map((node) => (node.id === nodeId ? { ...node, position } : node))
		};

		onstatechange?.(newState);
	}

	function handlePortMouseDown(event: CustomEvent) {
		const { nodeId, portId, portType } = event.detail;

		if (portType === 'output') {
			const newState = {
				...state,
				connectionState: {
					isConnecting: true,
					sourceNodeId: nodeId,
					sourcePortId: portId
				}
			};
			onstatechange?.(newState);
		}
	}

	function handlePortMouseUp(event: CustomEvent) {
		const { nodeId, portId, portType } = event.detail;

		if (
			state.connectionState.isConnecting &&
			portType === 'input' &&
			state.connectionState.sourceNodeId &&
			state.connectionState.sourcePortId
		) {
			const newConnection: ConnectionType = {
				id: generateId(),
				sourceNodeId: state.connectionState.sourceNodeId,
				sourcePortId: state.connectionState.sourcePortId,
				targetNodeId: nodeId,
				targetPortId: portId
			};

			const newState = {
				...state,
				connections: [...state.connections, newConnection],
				connectionState: {
					isConnecting: false
				}
			};

			onstatechange?.(newState);
		}
	}

	function handleSvgMouseMove(event: MouseEvent) {
		if (state.connectionState.isConnecting) {
			const mousePos = getMousePosition(event);
			// Update temp line for connection preview
			// This would be implemented with a temporary line component
		}
	}

	function handleSvgClick(event: MouseEvent) {
		if (state.connectionState.isConnecting) {
			// Cancel connection
			const newState = {
				...state,
				connectionState: { isConnecting: false }
			};
			onstatechange?.(newState);
		} else {
			// Clear selection
			const newState = {
				...state,
				selectedNodes: [],
				nodes: state.nodes.map((node) => ({ ...node, selected: false }))
			};
			onstatechange?.(newState);
		}
	}

	function handleZoom(delta: number) {
		const newScale = Math.max(0.1, Math.min(2, state.viewBox.scale + delta));
		const newState = {
			...state,
			viewBox: { ...state.viewBox, scale: newScale }
		};
		onstatechange?.(newState);
	}

	function handleNodeDoubleClick(event: CustomEvent) {
		const { nodeId } = event.detail;
		onnodeconfigopen?.({ nodeId });
	}

	function handleDeleteSelected() {
		const selectedIds = new Set(state.selectedNodes);
		const newState = {
			...state,
			nodes: state.nodes.filter((node) => !selectedIds.has(node.id)),
			connections: state.connections.filter(
				(conn) => !selectedIds.has(conn.sourceNodeId) && !selectedIds.has(conn.targetNodeId)
			),
			selectedNodes: []
		};
		onstatechange?.(newState);
	}

	// Keyboard shortcuts
	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Delete' || event.key === 'Backspace') {
			if (state.selectedNodes.length > 0) {
				handleDeleteSelected();
			}
		}
	}
</script>

<svelte:window on:keydown={handleKeyDown} />

<div bind:this={containerElement} class="flex h-full flex-col bg-gray-50 dark:bg-gray-900">
	<!-- 添加调试信息显示 -->
	{#if state.nodes.length > 0}
		<div class="absolute right-2 top-2 z-10 rounded bg-black bg-opacity-50 p-2 text-xs text-white">
			节点数量: {state.nodes.length}
		</div>
	{/if}

	<!-- Toolbar -->
	<div
		class="flex items-center justify-between border-b border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-800"
	>
		<div class="flex items-center space-x-2">
			<!-- Add Node Dropdown -->
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Button.Root
							{...props}
							class="flex items-center space-x-2 rounded-md bg-blue-600 px-3 py-2 text-white hover:bg-blue-700"
						>
							<Plus class="h-4 w-4" />
							<span>添加节点</span>
						</Button.Root>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Portal>
					<DropdownMenu.Content
						class="z-50 min-w-[160px] rounded-lg border border-gray-200 bg-white py-1 shadow-lg dark:border-gray-700 dark:bg-gray-800"
					>
						<DropdownMenu.Item
							class="flex cursor-pointer items-center px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700"
							onclick={() => handleAddNode('service')}
						>
							服务节点
						</DropdownMenu.Item>
						<DropdownMenu.Item
							class="flex cursor-pointer items-center px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700"
							onclick={() => handleAddNode('router')}
						>
							路由节点
						</DropdownMenu.Item>
						<DropdownMenu.Item
							class="flex cursor-pointer items-center px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700"
							onclick={() => handleAddNode('layer')}
						>
							层节点
						</DropdownMenu.Item>
						<DropdownMenu.Item
							class="flex cursor-pointer items-center px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700"
							onclick={() => handleAddNode('composition')}
						>
							组合节点
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Portal>
			</DropdownMenu.Root>
		</div>

		<div class="flex items-center space-x-2">
			<!-- Zoom controls -->
			<Button.Root
				class="rounded bg-gray-200 p-2 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600"
				onclick={() => handleZoom(-0.1)}
			>
				<ZoomOut class="h-4 w-4" />
			</Button.Root>

			<span class="text-sm text-gray-600 dark:text-gray-400">
				{Math.round(state.viewBox.scale * 100)}%
			</span>

			<Button.Root
				class="rounded bg-gray-200 p-2 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600"
				onclick={() => handleZoom(0.1)}
			>
				<ZoomIn class="h-4 w-4" />
			</Button.Root>

			<!-- Reset zoom -->
			<Button.Root
				class="rounded bg-gray-200 p-2 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600"
				onclick={() => handleZoom(1 - state.viewBox.scale)}
			>
				<RotateCcw class="h-4 w-4" />
			</Button.Root>

			<!-- Save/Load -->
			<Button.Root
				class="flex items-center space-x-2 rounded-md bg-green-600 px-3 py-2 text-white hover:bg-green-700"
				onclick={() => onsave?.(state)}
			>
				<Save class="h-4 w-4" />
				<span>保存</span>
			</Button.Root>
		</div>
	</div>
	<div class="bg-yellow-100 p-2 text-xs">
		<div>节点数量: {state.nodes.length}</div>
		<div>连接数量: {state.connections.length}</div>
		<div>
			ViewBox: {state.viewBox.x}, {state.viewBox.y}, {state.viewBox.width}, {state.viewBox.height}
		</div>
		<div>Scale: {state.viewBox.scale}</div>
		{#if state.nodes.length > 0}
			<div>第一个节点位置: x={state.nodes[0].position.x}, y={state.nodes[0].position.y}</div>
		{/if}
	</div>
	<!-- Canvas -->
	<div class="relative flex-1 overflow-hidden">
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<svg
			bind:this={svgElement}
			role="button"
			tabindex="0"
			class="h-full w-full"
			viewBox="{state.viewBox.x} {state.viewBox.y} {state.viewBox.width} {state.viewBox.height}"
			onclick={handleSvgClick}
			onmousemove={handleSvgMouseMove}
		>
			<!-- Define arrow marker -->
			<defs>
				<marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
					<polygon points="0 0, 10 3.5, 0 7" class="fill-gray-400" />
				</marker>
			</defs>

			<!-- Grid pattern -->
			<defs>
				<pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
					<path d="M 20 0 L 0 0 0 20" fill="none" stroke="#e5e7eb" stroke-width="1" />
				</pattern>
			</defs>
			<rect width="100%" height="100%" fill="url(#grid)" />

			<!-- Connections -->
			{#each state.connections as connection (connection.id)}
				<Connection {connection} nodes={state.nodes} />
			{/each}

			<!-- Nodes - 确保这部分代码正确 -->
			{#each state.nodes as node (node.id)}
				<!-- 添加调试矩形来验证节点位置 -->
				<rect
					x={node.position.x}
					y={node.position.y}
					width={node.size.width}
					height={node.size.height}
					fill="rgba(255, 0, 0, 0.2)"
					stroke="red"
					stroke-width="1"
				/>
				<text x={node.position.x + 10} y={node.position.y + 20} fill="red" font-size="12">
					{node.label}
				</text>
				<FlowNode
					{node}
					scale={state.viewBox.scale}
					on:nodeSelect={handleNodeSelect}
					on:nodeDrag={handleNodeDrag}
					on:portMouseDown={handlePortMouseDown}
					on:portMouseUp={handlePortMouseUp}
					on:nodeDoubleClick={handleNodeDoubleClick}
				/>
			{/each}

			<!-- Temporary connection line -->
			{#if state.connectionState.isConnecting && state.connectionState.tempLine}
				<line
					x1={state.connectionState.tempLine.start.x}
					y1={state.connectionState.tempLine.start.y}
					x2={state.connectionState.tempLine.end.x}
					y2={state.connectionState.tempLine.end.y}
					class="stroke-dasharray-5-5 stroke-blue-400 stroke-2"
				/>
			{/if}
		</svg>

		<!-- Node toolbar for selected nodes -->
		{#if state.selectedNodes.length > 0}
			<NodeToolbar selectedNodes={state.selectedNodes} on:delete={handleDeleteSelected} />
		{/if}
	</div>
</div>
