/**
 * Conversion utilities between FlowConfig and SvelteFlow format
 */

import type { Node, Edge } from '@xyflow/svelte';
import type { FlowConfig, InstanceData, InstanceDataWithoutType, NodeTarget } from './types';

/**
 * Convert FlowConfig to SvelteFlow nodes and edges
 */
export function flowToNodes(flow: FlowConfig): { nodes: Node[]; edges: Edge[] } {
	const nodes: Node[] = [];
	const edges: Edge[] = [];

	// Convert instances (with explicit type)
	Object.entries(flow.instances || {}).forEach(([id, instance]) => {
		nodes.push(createNodeFromInstance(id, instance, instance.type));
	});

	// Convert nodes (type = 'node')
	Object.entries(flow.nodes || {}).forEach(([id, nodeData]) => {
		nodes.push(createNodeFromInstance(id, nodeData, 'node'));
	});

	// Convert filters (type = 'filter')
	Object.entries(flow.filters || {}).forEach(([id, filterData]) => {
		nodes.push(createNodeFromInstance(id, filterData, 'filter'));
	});

	// Create edges from node configurations
	// For now, we'll just create a simple layout
	// TODO: Implement proper edge extraction from flow config

	return { nodes, edges };
}

function createNodeFromInstance(
	id: string,
	data: InstanceData | InstanceDataWithoutType,
	type: 'node' | 'filter'
): Node {
	return {
		id,
		type: 'custom', // We'll use custom nodes for all
		position: { x: 0, y: 0 }, // Will be laid out later
		data: {
			label: data.name || id,
			classId: data.class,
			instanceType: type,
			config: data.config
		}
	};
}

/**
 * Convert SvelteFlow nodes and edges back to FlowConfig
 */
export function nodesToFlow(nodes: Node[], edges: Edge[], entrypoint: NodeTarget): FlowConfig {
	const instances: Record<string, InstanceData> = {};
	const flowNodes: Record<string, InstanceDataWithoutType> = {};
	const filters: Record<string, InstanceDataWithoutType> = {};

	nodes.forEach((node) => {
		const instanceData: InstanceDataWithoutType = {
			name: node.data.label as string | undefined,
			class: node.data.classId as string,
			config: node.data.config || {}
		};

		const instanceType = node.data.instanceType || 'node';

		if (instanceType === 'node') {
			flowNodes[node.id] = instanceData;
		} else if (instanceType === 'filter') {
			filters[node.id] = instanceData;
		}
	});

	// TODO: Extract routing information from edges

	return {
		entrypoint,
		instances,
		nodes: flowNodes,
		filters,
		options: {}
	};
}

/**
 * Auto-layout nodes using a simple algorithm
 */
export function autoLayoutNodes(nodes: Node[]): Node[] {
	// Simple grid layout for now
	const columns = Math.ceil(Math.sqrt(nodes.length));
	const nodeWidth = 200;
	const nodeHeight = 100;
	const gap = 50;

	return nodes.map((node, index) => {
		const col = index % columns;
		const row = Math.floor(index / columns);

		return {
			...node,
			position: {
				x: col * (nodeWidth + gap),
				y: row * (nodeHeight + gap)
			}
		};
	});
}
