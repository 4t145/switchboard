import { getHttpClassEditorPlugin } from '$lib/plugins/registry';
import { isLinkValue } from '$lib/utils/link-parser';

export type NodeData = {
	class?: string;
	config: any;
};

export type TreeNode = {
	id: string;
	type: 'node' | 'filter';
	data: NodeData;
	children: TreeConnection[];
	isCycle?: boolean;
	isEntry?: boolean;
};

export type TreeConnection = {
	id: string;
	label: string;
	targetId?: string;
	filters: string[];
	targetNode?: TreeNode;
	filterNodes?: TreeNode[];
};

export type ResolveRequest = {
	nodeId: string;
	link: string;
	nodeType: 'node' | 'filter';
};

export type ResolveResult = {
	nodeId: string;
	success: boolean;
	data?: any;
	error?: string;
};

export type BatchResolveState = {
	isResolving: boolean;
	total: number;
	completed: number;
	failed: number;
	results: Map<string, any>;
};

export type BuildTreeWithResolveResult = {
	tree: TreeNode | null;
	orphans: TreeNode[];
	updatedNodes: Record<string, NodeData>;
	updatedFilters?: Record<string, NodeData>;
	resolveState: BatchResolveState;
};

function getOutputs(nodeId: string, node: NodeData) {
	if (!node.class && !node.config) return [];

	if (node.class) {
		const plugin = getHttpClassEditorPlugin(node.class, 'node');
		if (plugin?.extractOutputs && node.config) {
			try {
				const outputs = plugin.extractOutputs(node.config);
				if (outputs && outputs.length > 0) {
					return outputs;
				}
			} catch (e) {
				console.error(`Error extracting outputs for node ${nodeId}:`, e);
			}
		}
	}

	try {
		const config = node.config;
		if (!config) return [];

		if (typeof config === 'string') {
			const parsed = JSON.parse(config);
			if (parsed.outputs) return parsed.outputs;
		}

		if (typeof config === 'object') {
			if ('outputs' in config && Array.isArray(config.outputs)) {
				return config.outputs;
			}

			if ('targets' in config && Array.isArray(config.targets)) {
				return config.targets.map((target: any, index: number) => ({
					port: `output_${index}`,
					target: typeof target === 'string' ? target : target.id,
					label: target.label || `Output ${index + 1}`,
					filters: target.filters || []
				}));
			}

			const outputs: any[] = [];
			for (const key of Object.keys(config)) {
				const value = config[key];
				if (value && typeof value === 'object') {
					if ('target' in value || 'to' in value) {
						outputs.push({
							port: key,
							target: value.target || value.to,
							label: value.label || key,
							filters: value.filters || []
						});
					}
				}
			}

			if (outputs.length > 0) return outputs;
		}
	} catch (e) {
		console.warn(`Could not parse outputs for node ${nodeId}:`, e);
	}

	return [];
}

export function identifyLinksToResolve(
	nodes: Record<string, NodeData>,
	filters: Record<string, NodeData>
): ResolveRequest[] {
	const requests: ResolveRequest[] = [];

	for (const [nodeId, node] of Object.entries(nodes)) {
		const config = node.config;
		if (typeof config === 'string' && isLinkValue(config)) {
			requests.push({
				nodeId,
				link: config,
				nodeType: 'node'
			});
		}
	}

	for (const [filterId, filter] of Object.entries(filters)) {
		const config = filter.config;
		if (typeof config === 'string' && isLinkValue(config)) {
			requests.push({
				nodeId: filterId,
				link: config,
				nodeType: 'filter'
			});
		}
	}

	return requests;
}

export async function batchResolveLinks(
	requests: ResolveRequest[],
	onProgress?: (state: BatchResolveState) => void
): Promise<Map<string, any>> {
	const results = new Map<string, any>();
	let completed = 0;

	for (const request of requests) {
		try {
			const response = await fetch('/api/resolve/link_to_object', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ link: request.link })
			});

			if (!response.ok) {
				throw new Error(`HTTP ${response.status}: ${response.statusText}`);
			}

			const data = await response.json();
			results.set(request.nodeId, data);
			completed++;

			if (onProgress) {
				onProgress({
					isResolving: true,
					total: requests.length,
					completed,
					failed: 0,
					results
				});
			}
		} catch (e) {
			console.error(`Failed to resolve ${request.nodeId}:`, e);
			completed++;

			const failedCount = Array.from(results.values()).filter(
				(r, i) => i >= completed && !r
			).length;

			if (onProgress) {
				onProgress({
					isResolving: true,
					total: requests.length,
					completed,
					failed: failedCount + 1,
					results
				});
			}
		}
	}

	return results;
}

export function buildFlowTree(
	nodes: Record<string, NodeData>,
	filters: Record<string, NodeData>,
	entrypoint: string
): { tree: TreeNode | null; orphans: TreeNode[] } {
	const visitedInPath = new Set<string>();
	const allVisited = new Set<string>();

	function buildNode(id: string, isEntry: boolean = false): TreeNode {
		const isCycle = visitedInPath.has(id);
		allVisited.add(id);

		const nodeData = nodes[id];
		if (!nodeData) {
			return {
				id,
				type: 'node',
				data: { config: {} },
				children: [],
				isCycle: false
			};
		}

		if (isCycle) {
			return {
				id,
				type: 'node',
				data: nodeData,
				children: [],
				isCycle: true,
				isEntry
			};
		}

		visitedInPath.add(id);

		const outputs = getOutputs(id, nodeData);
		const children: TreeConnection[] = outputs.map((output) => {
			const connectionId = `${id}-${output.port}`;

			const filterNodes: TreeNode[] = [];
			if (output.filters) {
				output.filters.forEach((filterId) => {
					if (filters[filterId]) {
						filterNodes.push({
							id: filterId,
							type: 'filter',
							data: filters[filterId],
							children: []
						});
					}
				});
			}

			let targetNode: TreeNode | undefined;
			if (output.target) {
				targetNode = buildNode(output.target);
			}

			return {
				id: connectionId,
				label: output.label || output.port,
				targetId: output.target,
				filters: output.filters || [],
				filterNodes,
				targetNode
			};
		});

		visitedInPath.delete(id);

		return {
			id,
			type: 'node',
			data: nodeData,
			children,
			isCycle: false,
			isEntry
		};
	}

	const tree = entrypoint ? buildNode(entrypoint, true) : null;

	const orphans: TreeNode[] = [];
	Object.keys(nodes).forEach((nodeId) => {
		if (!allVisited.has(nodeId)) {
			orphans.push({
				id: nodeId,
				type: 'node',
				data: nodes[nodeId],
				children: [],
				isCycle: false
			});
		}
	});

	return { tree, orphans };
}

export async function buildFlowTreeWithResolve(
	nodes: Record<string, NodeData>,
	filters: Record<string, NodeData>,
	entrypoint: string,
	onResolveProgress?: (state: BatchResolveState) => void
): Promise<BuildTreeWithResolveResult> {
	onResolveProgress?.({
		isResolving: true,
		total: 0,
		completed: 0,
		failed: 0,
		results: new Map()
	});

	const requests = identifyLinksToResolve(nodes, filters);

	onResolveProgress?.({
		isResolving: true,
		total: requests.length,
		completed: 0,
		failed: 0,
		results: new Map()
	});

	let resolved = new Map<string, any>();

	if (requests.length > 0) {
		resolved = await batchResolveLinks(requests, onResolveProgress);
	}

	const updatedNodes: Record<string, NodeData> = {
		...nodes,
		...filters
	};

	for (const [nodeId, config] of resolved) {
		if (updatedNodes[nodeId]) {
			updatedNodes[nodeId] = { ...updatedNodes[nodeId], config };
		}
	}

	onResolveProgress?.({
		isResolving: false,
		total: requests.length,
		completed: requests.length,
		failed: 0,
		results: resolved
	});

	const { tree, orphans } = buildFlowTree(updatedNodes, filters, entrypoint);

	return { tree, orphans, updatedNodes, resolveState: { isResolving: false, total: requests.length, completed: requests.length, failed: 0, results: resolved } };
}
